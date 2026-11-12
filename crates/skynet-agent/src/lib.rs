//! `skynet-agent`: Complete Multicellular AI `AgentNode` combining `candle` micro-cells,
//! opportunistic zero-impact GPU/CPU harvesting, P2P NAT rendezvous, 3-tier workload
//! multiplexing, PoUW Canary spot-checks, and loss-tolerant Snowball consensus over UDP.

use std::collections::{HashMap, HashSet};

use ed25519_dalek::SigningKey;
use skynet_cell::{CellError, HardwareTier, MicroCell};
use skynet_dna::CellRole;
use skynet_ledger::{AgentMetabolicBank, LedgerError, NanoSkn, UtxoDagLedger, UtxoId};
use skynet_wire::WireError;
use thiserror::Error;

pub mod auditor_flow;
pub mod compute_flow;
pub mod datagram_router;
pub mod governor_fsm;
pub mod opportunistic_worker;
pub mod organism_state;
pub mod rendezvous_broker;
pub mod snowball;
pub mod workload_mux;

pub use auditor_flow::AuditorCoordinator;
pub use compute_flow::{AssignTaskParams, ComputeFlowCoordinator, ComputeFlowError, PrepareTaskParams};
pub use governor_fsm::{
    GovernorConfig, GovernorFsm, GovernorState, SystemTelemetrySnapshot, ACTIVE_POLL_INTERVAL_MS,
    DEFAULT_HYSTERESIS_COOL_C, DEFAULT_IDLE_HOLD_SECS, DEFAULT_LIGHT_IDLE_UTIL_PCT,
    DEFAULT_MAX_VRAM_PCT, DEFAULT_TEMP_LIMIT_C, DEFAULT_THROTTLE_MARGIN_C, PAUSED_POLL_INTERVAL_MS,
};
pub use opportunistic_worker::{
    OpportunisticWorkerEngine, OpportunisticWorkerError, ShardExecutionStep, MAX_IN_FLIGHT_SHARDS,
};
pub use organism_state::AgentNode;
pub use rendezvous_broker::{
    PeerRendezvousEntry, RelayRouteDecision, RendezvousError, RendezvousPairCommands,
    RendezvousRegistry, HOLE_PUNCH_BURST_COUNT, RENDEZVOUS_TTL_SEC, SEED_KEEPALIVE_INTERVAL_SEC,
};
pub use snowball::{SnowballEngine, SnowballParams};
pub use workload_mux::{
    PowerProfitabilityGuard, ProfitabilityVerdict, QueuedWorkload, WorkloadClass, WorkloadDecision,
    WorkloadMultiplexer, WorkloadMuxError,
};

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("cell error: {0}")]
    Cell(#[from] CellError),
    #[error("ledger error: {0}")]
    Ledger(#[from] LedgerError),
    #[error("wire error: {0}")]
    Wire(#[from] WireError),
    #[error("compute flow error: {0}")]
    Compute(#[from] ComputeFlowError),
    #[error("rendezvous error: {0}")]
    Rendezvous(#[from] RendezvousError),
    #[error("opportunistic worker error: {0}")]
    Opportunistic(#[from] OpportunisticWorkerError),
    #[error("UDP I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Summary snapshot of an Agent's multicellular body.
#[derive(Debug, Clone)]
pub struct OrganismStats {
    pub name: String,
    pub tier: HardwareTier,
    pub cluster_prefix: u32,
    pub prefix_len: u8,
    pub total_cells: usize,
    pub role_counts: HashMap<CellRole, usize>,
    pub max_generation: u64,
    pub on_chain_balance: NanoSkn,
    pub blood_pool: NanoSkn,
    pub total_mitosis_events: u64,
    pub total_apoptosis_events: u64,
    pub udp_sent: u64,
    pub udp_received: u64,
    pub udp_dropped_simulated: u64,
}

pub type PendingSnowballQuery = (u64, HashSet<[u8; 32]>, Vec<UtxoId>);

/// Parameters for restoring a persisted multicellular organism.
#[derive(Debug)]
pub struct RestoreOrganismOptions {
    pub name: String,
    pub seed: u64,
    pub signing_key: SigningKey,
    pub tier: HardwareTier,
    pub ledger: UtxoDagLedger,
    pub cells: Vec<MicroCell>,
    pub bank: AgentMetabolicBank,
    pub snowball_params: SnowballParams,
}

impl AgentNode {
    pub fn stats(&self) -> OrganismStats {
        let (mut role_counts, mut max_gen) = (HashMap::new(), 0u64);
        for c in &self.cells {
            *role_counts.entry(c.role()).or_default() += 1;
            max_gen = max_gen.max(c.dna.generation);
        }
        OrganismStats {
            name: self.name.clone(), tier: self.profile.tier, cluster_prefix: self.cluster_prefix,
            prefix_len: self.prefix_len, total_cells: self.cells.len(), role_counts, max_generation: max_gen,
            on_chain_balance: self.ledger.balance_of(&self.pubkey), blood_pool: self.bank.blood_pool,
            total_mitosis_events: self.total_mitosis_events, total_apoptosis_events: self.total_apoptosis_events,
            udp_sent: self.udp_sent, udp_received: self.udp_received, udp_dropped_simulated: self.udp_dropped_simulated,
        }
    }
}

#[cfg(test)]
mod agent_core_tests;
#[cfg(test)]
mod agent_network_tests;
