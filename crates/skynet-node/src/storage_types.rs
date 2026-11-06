//! Serializable snapshot data transfer objects (DTOs) for node persistence.

use serde::{Deserialize, Serialize};
use skynet_cell::{HardwareTier, MicroCell};
use skynet_ledger::{NanoSkn, TxOutput, UtxoId};
use skynet_wire::PeerContact;

/// Persistent metadata of the node daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeMetadata {
    pub name: String,
    pub tier: HardwareTier,
    pub pubkey: [u8; 32],
    pub epoch: u64,
    pub pulse_seq: u32,
    pub created_at_secs: u64,
    pub last_saved_secs: u64,
}

/// Snapshot of the UTXO-DAG ledger state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerSnapshot {
    pub utxos: Vec<(UtxoId, TxOutput)>,
    pub finalized_tips: Vec<[u8; 32]>,
    pub cluster_nutrient_pool: NanoSkn,
}

/// Snapshot of the biological cells and internal metabolic bank of an Agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganismSnapshot {
    pub cells: Vec<MicroCell>,
    pub blood_pool: NanoSkn,
    pub total_voucher_nano_earned: NanoSkn,
}

/// Snapshot of discovered network peers in the synaptic routing table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerTableSnapshot {
    pub peers: Vec<PeerContact>,
}

/// Consolidated state container representing a full node snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullNodeSnapshot {
    pub metadata: NodeMetadata,
    pub ledger: LedgerSnapshot,
    pub organism: OrganismSnapshot,
    pub peer_table: PeerTableSnapshot,
}
