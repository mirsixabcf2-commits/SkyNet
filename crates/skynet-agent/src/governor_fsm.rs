//! `governor_fsm`: Pure zero-impact hardware governor state machine driving `PreemptGuard`
//! with `8°C` thermal anti-flap hysteresis and `30s` continuous idle recovery hold.

use serde::{Deserialize, Serialize};
use skynet_cell::{PreemptGuard, PreemptSignal};

pub const DEFAULT_TEMP_LIMIT_C: u8 = 78;
pub const DEFAULT_HYSTERESIS_COOL_C: u8 = 8;
pub const DEFAULT_THROTTLE_MARGIN_C: u8 = 5;
pub const DEFAULT_IDLE_HOLD_SECS: u64 = 30;
pub const DEFAULT_MAX_VRAM_PCT: u8 = 50;
pub const DEFAULT_LIGHT_IDLE_UTIL_PCT: u8 = 25;
pub const ACTIVE_POLL_INTERVAL_MS: u64 = 250;
pub const PAUSED_POLL_INTERVAL_MS: u64 = 1_500;
const MINUTES_PER_DAY: u64 = 1_440;

/// User/Production policy governing zero-impact opportunistic GPU/CPU harvesting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernorConfig {
    pub temp_limit_c: u8,
    pub hysteresis_cool_c: u8,
    pub idle_hold_secs: u64,
    pub max_vram_pct: u8,
    pub battery_guard: bool,
    /// Optional allowed active window in minutes-from-midnight `(start_min, end_min)`.
    pub schedule_window: Option<(u16, u16)>,
}

impl Default for GovernorConfig {
    fn default() -> Self {
        Self {
            temp_limit_c: DEFAULT_TEMP_LIMIT_C,
            hysteresis_cool_c: DEFAULT_HYSTERESIS_COOL_C,
            idle_hold_secs: DEFAULT_IDLE_HOLD_SECS,
            max_vram_pct: DEFAULT_MAX_VRAM_PCT,
            battery_guard: true,
            schedule_window: None,
        }
    }
}

impl GovernorConfig {
    pub fn cool_resume_temp_c(&self) -> u8 {
        self.temp_limit_c.saturating_sub(self.hysteresis_cool_c)
    }

    pub fn throttle_temp_c(&self) -> u8 {
        self.temp_limit_c.saturating_sub(DEFAULT_THROTTLE_MARGIN_C)
    }

    pub fn is_within_schedule(&self, observed_at_sec: u64) -> bool {
        let Some((start_min, end_min)) = self.schedule_window else {
            return true;
        };
        if start_min == end_min {
            return true;
        }
        let minute = ((observed_at_sec / 60) % MINUTES_PER_DAY) as u16;
        if start_min < end_min {
            minute >= start_min && minute < end_min
        } else {
            minute >= start_min || minute < end_min
        }
    }
}

/// Point-in-time rootless OS telemetry sample consumed by `GovernorFsm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemTelemetrySnapshot {
    pub gpu_util_pct: u8,
    pub cpu_util_pct: u8,
    pub vram_used_pct: u8,
    pub temp_c: u8,
    pub on_battery: bool,
    pub active_game_detected: bool,
    pub observed_at_sec: u64,
}

impl Default for SystemTelemetrySnapshot {
    fn default() -> Self {
        Self {
            gpu_util_pct: 5, cpu_util_pct: 5, vram_used_pct: 15, temp_c: 45,
            on_battery: false, active_game_detected: false, observed_at_sec: 0,
        }
    }
}

/// Operational tier of the opportunistic edge worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernorState {
    DeepIdle,
    LightIdle,
    Throttled,
    PausedGaming,
    PausedThermal,
    PausedBattery,
    PausedVram,
    PausedSchedule,
}

impl GovernorState {
    pub fn is_paused(self) -> bool {
        matches!(
            self,
            Self::PausedGaming | Self::PausedThermal | Self::PausedBattery | Self::PausedVram | Self::PausedSchedule
        )
    }

    pub fn preempt_signal(self) -> PreemptSignal {
        match self {
            Self::DeepIdle | Self::LightIdle => PreemptSignal::Run,
            Self::Throttled => PreemptSignal::Throttle50,
            _ => PreemptSignal::PreemptImmediate,
        }
    }

    pub fn recommended_poll_ms(self) -> u64 {
        if self.is_paused() { PAUSED_POLL_INTERVAL_MS } else { ACTIVE_POLL_INTERVAL_MS }
    }
}

/// Pure deterministic finite-state machine updating `PreemptGuard` from `SystemTelemetrySnapshot`.
#[derive(Debug, Clone)]
pub struct GovernorFsm {
    pub config: GovernorConfig,
    state: GovernorState,
    idle_recovery_started_sec: Option<u64>,
}

impl Default for GovernorFsm {
    fn default() -> Self { Self::new(GovernorConfig::default()) }
}

impl GovernorFsm {
    pub fn new(config: GovernorConfig) -> Self {
        Self { config, state: GovernorState::DeepIdle, idle_recovery_started_sec: None }
    }

    pub fn state(&self) -> GovernorState { self.state }

    pub fn idle_hold_elapsed_secs(&self, now_sec: u64) -> u64 {
        self.idle_recovery_started_sec.map(|s| now_sec.saturating_sub(s)).unwrap_or(0)
    }

    /// Evaluates `snap`, enforces `8°C` hysteresis & `30s` hold, and updates `guard` atomically.
    pub fn evaluate(&mut self, snap: &SystemTelemetrySnapshot, guard: &PreemptGuard) -> GovernorState {
        if let Some(hard_pause) = detect_hard_pause(&self.config, snap) {
            self.idle_recovery_started_sec = None;
            return self.commit_state(hard_pause, guard);
        }
        if self.state.is_paused() {
            if snap.temp_c > self.config.cool_resume_temp_c() {
                self.idle_recovery_started_sec = None;
                return self.commit_state(self.state, guard);
            }
            let start = *self.idle_recovery_started_sec.get_or_insert(snap.observed_at_sec);
            if snap.observed_at_sec.saturating_sub(start) < self.config.idle_hold_secs {
                return self.commit_state(self.state, guard);
            }
            self.idle_recovery_started_sec = None;
        }
        self.commit_state(classify_active_tier(&self.config, snap), guard)
    }

    fn commit_state(&mut self, next: GovernorState, guard: &PreemptGuard) -> GovernorState {
        self.state = next;
        guard.set_signal(next.preempt_signal());
        next
    }
}

fn detect_hard_pause(cfg: &GovernorConfig, snap: &SystemTelemetrySnapshot) -> Option<GovernorState> {
    if snap.active_game_detected { return Some(GovernorState::PausedGaming); }
    if cfg.battery_guard && snap.on_battery { return Some(GovernorState::PausedBattery); }
    if snap.temp_c >= cfg.temp_limit_c { return Some(GovernorState::PausedThermal); }
    if snap.vram_used_pct > cfg.max_vram_pct { return Some(GovernorState::PausedVram); }
    if !cfg.is_within_schedule(snap.observed_at_sec) { return Some(GovernorState::PausedSchedule); }
    None
}

fn classify_active_tier(cfg: &GovernorConfig, snap: &SystemTelemetrySnapshot) -> GovernorState {
    if snap.temp_c >= cfg.throttle_temp_c() { return GovernorState::Throttled; }
    if snap.gpu_util_pct >= DEFAULT_LIGHT_IDLE_UTIL_PCT || snap.cpu_util_pct >= DEFAULT_LIGHT_IDLE_UTIL_PCT {
        return GovernorState::LightIdle;
    }
    GovernorState::DeepIdle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thermal_hysteresis_8c_and_30s_hold_prevent_flapping() {
        let (mut fsm, guard) = (GovernorFsm::new(GovernorConfig::default()), PreemptGuard::new());
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { temp_c: 78, observed_at_sec: 10, ..Default::default() }, &guard), GovernorState::PausedThermal);
        assert_eq!(guard.signal(), PreemptSignal::PreemptImmediate);
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { temp_c: 75, observed_at_sec: 60, ..Default::default() }, &guard), GovernorState::PausedThermal);
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { temp_c: 70, observed_at_sec: 70, ..Default::default() }, &guard), GovernorState::PausedThermal);
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { temp_c: 70, observed_at_sec: 80, ..Default::default() }, &guard), GovernorState::PausedThermal);
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { temp_c: 70, observed_at_sec: 100, ..Default::default() }, &guard), GovernorState::DeepIdle);
        assert_eq!(guard.signal(), PreemptSignal::Run);
    }

    #[test]
    fn gaming_battery_vram_throttle_light_and_schedule_transitions() {
        let (mut fsm, guard) = (GovernorFsm::new(GovernorConfig { schedule_window: Some((60, 1200)), ..Default::default() }), PreemptGuard::new());
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { observed_at_sec: 100, ..Default::default() }, &guard), GovernorState::PausedSchedule);
        let t = 3_600u64;
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { active_game_detected: true, observed_at_sec: t, ..Default::default() }, &guard), GovernorState::PausedGaming);
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { on_battery: true, observed_at_sec: t + 5, ..Default::default() }, &guard), GovernorState::PausedBattery);
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { vram_used_pct: 55, observed_at_sec: t + 10, ..Default::default() }, &guard), GovernorState::PausedVram);
        let _ = fsm.evaluate(&SystemTelemetrySnapshot { observed_at_sec: t + 20, ..Default::default() }, &guard);
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { gpu_util_pct: 35, observed_at_sec: t + 50, ..Default::default() }, &guard), GovernorState::LightIdle);
        assert_eq!(fsm.evaluate(&SystemTelemetrySnapshot { temp_c: 74, observed_at_sec: t + 55, ..Default::default() }, &guard), GovernorState::Throttled);
        assert_eq!(guard.signal(), PreemptSignal::Throttle50);
    }
}
