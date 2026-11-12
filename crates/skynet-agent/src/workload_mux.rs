//! `workload_mux`: 3-tier `WorkloadMultiplexer` interleaving `< 10ms` fast inference with
//! micro-training/plasmid evolution and `PowerProfitabilityGuard` electricity cost floor.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use skynet_ledger::NanoSkn;
use skynet_wire::TaskId;
use thiserror::Error;

use crate::governor_fsm::GovernorState;

const MAX_MUX_QUEUE_LEN: usize = 64;
const BPS_DENOM: u128 = 10_000;
const WATT_SECONDS_PER_KWH: u128 = 3_600_000;
const MILLIS_PER_SEC: u128 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkloadClass {
    MicroTrainingShard,
    PlasmidEvolution,
    FastInferenceSub10ms,
    CanaryVerification,
}

impl WorkloadClass {
    pub fn is_latency_priority(self) -> bool {
        matches!(self, Self::FastInferenceSub10ms | Self::CanaryVerification)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkloadDecision {
    AdmitFullSpeed,
    AdmitPriorityInterleaved,
    AdmitThrottled50,
    Rejected { state: GovernorState, class: WorkloadClass },
}

impl WorkloadDecision {
    pub fn is_admitted(self) -> bool { !matches!(self, Self::Rejected { .. }) }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum WorkloadMuxError {
    #[error("workload {class:?} rejected under governor state {state:?}")]
    RejectedByGovernor { state: GovernorState, class: WorkloadClass },
    #[error("bid unprofitable: offered {offered_nano_per_sec} < required {required_nano_per_sec} nanoSKN/s")]
    UnprofitableBid { offered_nano_per_sec: u128, required_nano_per_sec: u128 },
    #[error("workload multiplexer queue capacity ({MAX_MUX_QUEUE_LEN}) exceeded")]
    QueueFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueuedWorkload {
    pub task_id: TaskId,
    pub class: WorkloadClass,
}

/// 3-tier scheduler coordinating `DeepIdle`, `LightIdle`/`Throttled`, and `Paused*` execution.
#[derive(Debug, Clone, Default)]
pub struct WorkloadMultiplexer {
    fast_lane: VecDeque<QueuedWorkload>,
    background_lane: VecDeque<QueuedWorkload>,
}

impl WorkloadMultiplexer {
    pub fn new() -> Self { Self::default() }

    pub fn allows_class(state: GovernorState, class: WorkloadClass) -> bool {
        Self::classify_decision(state, class).is_admitted()
    }

    pub fn classify_decision(state: GovernorState, class: WorkloadClass) -> WorkloadDecision {
        if state.is_paused() {
            return WorkloadDecision::Rejected { state, class };
        }
        match (state, class.is_latency_priority()) {
            (GovernorState::DeepIdle | GovernorState::Throttled, true) => WorkloadDecision::AdmitPriorityInterleaved,
            (GovernorState::DeepIdle, false) => WorkloadDecision::AdmitFullSpeed,
            (GovernorState::LightIdle, true) => WorkloadDecision::AdmitFullSpeed,
            (GovernorState::Throttled, false) => WorkloadDecision::AdmitThrottled50,
            _ => WorkloadDecision::Rejected { state, class },
        }
    }

    pub fn evaluate_admission(&self, state: GovernorState, class: WorkloadClass) -> WorkloadDecision {
        Self::classify_decision(state, class)
    }

    pub fn ensure_admitted(&self, state: GovernorState, class: WorkloadClass) -> Result<WorkloadDecision, WorkloadMuxError> {
        let decision = Self::classify_decision(state, class);
        if !decision.is_admitted() {
            return Err(WorkloadMuxError::RejectedByGovernor { state, class });
        }
        Ok(decision)
    }

    pub fn pending_count(&self) -> usize { self.fast_lane.len() + self.background_lane.len() }

    pub fn enqueue(&mut self, item: QueuedWorkload, state: GovernorState) -> Result<WorkloadDecision, WorkloadMuxError> {
        let decision = self.ensure_admitted(state, item.class)?;
        if self.pending_count() >= MAX_MUX_QUEUE_LEN {
            return Err(WorkloadMuxError::QueueFull);
        }
        if item.class.is_latency_priority() {
            self.fast_lane.push_back(item);
        } else {
            self.background_lane.push_back(item);
        }
        Ok(decision)
    }

    pub fn pop_next_runnable(&mut self, state: GovernorState) -> Option<QueuedWorkload> {
        if state.is_paused() {
            return None;
        }
        if let Some(fast) = self.fast_lane.pop_front() {
            return Some(fast);
        }
        if matches!(state, GovernorState::DeepIdle | GovernorState::Throttled) {
            return self.background_lane.pop_front();
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfitabilityVerdict {
    pub offered_nano_per_sec: u128,
    pub required_nano_per_sec: u128,
    pub net_margin_nano_per_sec: u128,
}

/// Rejects task bids whose `nanoSKN/sec` reward falls below estimated device electricity cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerProfitabilityGuard {
    pub estimated_tdp_watts: u16,
    pub min_profitable_nano_per_sec: u128,
    pub margin_bps: u16,
}

impl Default for PowerProfitabilityGuard {
    fn default() -> Self {
        Self { estimated_tdp_watts: 120, min_profitable_nano_per_sec: 1_000, margin_bps: 11_000 }
    }
}

impl PowerProfitabilityGuard {
    pub fn from_power_cost(estimated_tdp_watts: u16, nano_skn_per_kwh: u128, margin_bps: u16) -> Self {
        let floor = u128::from(estimated_tdp_watts).saturating_mul(nano_skn_per_kwh).div_ceil(WATT_SECONDS_PER_KWH);
        Self { estimated_tdp_watts, min_profitable_nano_per_sec: floor, margin_bps: margin_bps.max(10_000) }
    }

    pub const fn with_floor_nano_per_sec(min_profitable_nano_per_sec: u128) -> Self {
        Self { estimated_tdp_watts: 100, min_profitable_nano_per_sec, margin_bps: 10_000 }
    }

    pub fn required_nano_per_sec(&self) -> u128 {
        self.min_profitable_nano_per_sec.saturating_mul(u128::from(self.margin_bps.max(10_000))).div_ceil(BPS_DENOM)
    }

    pub fn is_rate_profitable(&self, offered_nano_per_sec: u128) -> bool {
        offered_nano_per_sec >= self.required_nano_per_sec()
    }

    pub fn evaluate_bid(&self, offered_bounty: NanoSkn, estimated_duration_ms: u64) -> Result<ProfitabilityVerdict, WorkloadMuxError> {
        let offered = offered_bounty.0.saturating_mul(MILLIS_PER_SEC) / u128::from(estimated_duration_ms.max(1));
        let required = self.required_nano_per_sec();
        if offered < required {
            return Err(WorkloadMuxError::UnprofitableBid { offered_nano_per_sec: offered, required_nano_per_sec: required });
        }
        Ok(ProfitabilityVerdict { offered_nano_per_sec: offered, required_nano_per_sec: required, net_margin_nano_per_sec: offered - required })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_tier_mux_interleaves_fast_inference_and_enforces_light_and_paused_rules() {
        let mut mux = WorkloadMultiplexer::new();
        mux.enqueue(QueuedWorkload { task_id: [1; 32], class: WorkloadClass::MicroTrainingShard }, GovernorState::DeepIdle).unwrap();
        mux.enqueue(QueuedWorkload { task_id: [2; 32], class: WorkloadClass::PlasmidEvolution }, GovernorState::DeepIdle).unwrap();
        mux.enqueue(QueuedWorkload { task_id: [3; 32], class: WorkloadClass::FastInferenceSub10ms }, GovernorState::DeepIdle).unwrap();

        assert_eq!(mux.pop_next_runnable(GovernorState::DeepIdle).unwrap().task_id, [3; 32]);
        assert!(mux.enqueue(QueuedWorkload { task_id: [4; 32], class: WorkloadClass::MicroTrainingShard }, GovernorState::LightIdle).is_err());
        mux.enqueue(QueuedWorkload { task_id: [5; 32], class: WorkloadClass::CanaryVerification }, GovernorState::LightIdle).unwrap();
        assert_eq!(mux.pop_next_runnable(GovernorState::LightIdle).unwrap().task_id, [5; 32]);
        assert_eq!(mux.pop_next_runnable(GovernorState::LightIdle), None);
        assert_eq!(mux.pop_next_runnable(GovernorState::PausedGaming), None);
        assert!(mux.enqueue(QueuedWorkload { task_id: [6; 32], class: WorkloadClass::FastInferenceSub10ms }, GovernorState::PausedThermal).is_err());
        assert_eq!(mux.pop_next_runnable(GovernorState::DeepIdle).unwrap().task_id, [1; 32]);
    }

    #[test]
    fn power_profitability_guard_rejects_bids_below_electricity_cost() {
        let guard = PowerProfitabilityGuard::from_power_cost(200, 360_000_000, 11_000);
        assert_eq!(guard.required_nano_per_sec(), 22_000);
        assert!(guard.evaluate_bid(NanoSkn(10_000), 500).is_err());
        let ok = guard.evaluate_bid(NanoSkn(25_000), 1_000).unwrap();
        assert_eq!(ok.net_margin_nano_per_sec, 3_000);
    }
}
