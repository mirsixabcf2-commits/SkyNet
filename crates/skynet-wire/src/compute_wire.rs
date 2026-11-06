//! `compute_wire`: Wire definitions and serialization helpers for distributed compute tasks
//! over UDP impulse datagrams (`<= 1200 bytes`).

use serde::{Deserialize, Serialize};

use crate::{FountainDecoder, FountainDroplet, FountainEncoder, WireError};

/// 32-byte cryptographic identifier for a remote compute task.
pub type TaskId = [u8; 32];

/// Core specification of a remote compute task broadcast over the network.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeTaskManifest {
    pub task_id: TaskId,
    pub client_pubkey: [u8; 32],
    pub opcode: u16,
    pub required_tier: u8,
    pub input_hash: [u8; 32],
    pub timeout_epoch: u64,
    pub max_ticks: u64,
    pub bounty_nano_skn: u128,
    pub min_worker_bond_nano_skn: u128,
    pub auditor_bounty_nano_skn: u128,
    pub required_auditors: u8,
}

/// Bid message submitted by a candidate worker node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeBid {
    pub task_id: TaskId,
    pub worker_pubkey: [u8; 32],
    pub bond_nano_skn: u128,
    pub worker_tier: u8,
}

/// Bid message submitted by a candidate auditor node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeAuditorBid {
    pub task_id: TaskId,
    pub auditor_pubkey: [u8; 32],
    pub bond_nano_skn: u128,
    pub auditor_signature: Vec<u8>,
}

/// Task assignment issued by the client selecting an authorized worker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeAssignment {
    pub task_id: TaskId,
    pub worker_pubkey: [u8; 32],
    pub client_signature: Vec<u8>,
}

/// Completion claim published by the worker with output digest and execution receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeCompletionClaim {
    pub task_id: TaskId,
    pub worker_pubkey: [u8; 32],
    pub result_digest: [u8; 32],
    pub execution_ticks: u64,
    pub worker_signature: Vec<u8>,
}

/// Signed attestation from a community auditor verifying task output digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeAuditorAttestation {
    pub task_id: TaskId,
    pub auditor_pubkey: [u8; 32],
    pub result_digest: [u8; 32],
    pub auditor_signature: Vec<u8>,
}

/// Enum of all compute-related message payloads transported inside `ImpulsePacket`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeMessage {
    /// Broadcast announcement of a compute task seeking workers.
    TaskAnnouncement(ComputeTaskManifest),
    /// Worker bid proposing execution capability and staking commitment.
    TaskBid(ComputeBid),
    /// Auditor bid proposing verification capability and staking commitment.
    AuditorBid(ComputeAuditorBid),
    /// Client confirmation assigning the task to a specific worker.
    TaskAssigned(ComputeAssignment),
    /// Compact direct payload (`<= 800B`) for micro-neural inputs or outputs.
    DirectPayload {
        task_id: TaskId,
        is_result: bool,
        data: Vec<u8>,
    },
    /// Erasure-coded Fountain droplet for larger inputs or model checkpoints.
    FountainPayload {
        task_id: TaskId,
        is_result: bool,
        droplet: FountainDroplet,
    },
    /// Formal claim of task completion triggering escrow settlement.
    CompletionClaim(ComputeCompletionClaim),
    /// Auditor attestation certifying result integrity for quorum settlement.
    AuditorAttestation(ComputeAuditorAttestation),
}

/// Helper converting a large compute payload into a stream of erasure-coded droplets.
pub fn encode_compute_payload(payload: &[u8]) -> Result<FountainEncoder, WireError> {
    FountainEncoder::new(payload)
}

/// Helper attempting reconstruction of a compute payload from received droplets.
pub fn try_decode_compute_payload(
    decoder: &mut FountainDecoder,
    droplet: FountainDroplet,
) -> Result<Option<Vec<u8>>, WireError> {
    decoder.absorb(droplet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_manifest_serialization_compactness() {
        let manifest = ComputeTaskManifest {
            task_id: [1u8; 32],
            client_pubkey: [2u8; 32],
            opcode: 0x1101, // AI Micro-inference
            required_tier: 2,
            input_hash: [3u8; 32],
            timeout_epoch: 100,
            max_ticks: 50_000,
            bounty_nano_skn: 10_000,
            min_worker_bond_nano_skn: 2_000,
            auditor_bounty_nano_skn: 1_500,
            required_auditors: 3,
        };
        let msg = ComputeMessage::TaskAnnouncement(manifest);
        let encoded = bincode::serialize(&msg).expect("serialization succeeds");
        assert!(
            encoded.len() < 250,
            "manifest must fit well inside UDP MTU"
        );
    }

    #[test]
    fn test_compute_payload_fountain_droplets_under_packet_loss() {
        // Create 4KB dummy tensor data
        let mut original_tensor = Vec::with_capacity(4096);
        for i in 0..1024 {
            original_tensor.extend_from_slice(&(i as f32).to_le_bytes());
        }

        let mut encoder = encode_compute_payload(&original_tensor).expect("encoder created");
        let first_droplet = encoder.next_droplet();
        let mut decoder = FountainDecoder::from_first_droplet(&first_droplet);

        // Feed first droplet
        let mut result = try_decode_compute_payload(&mut decoder, first_droplet).unwrap();

        // Simulate 35% packet loss: skip droplets where seq % 3 == 0
        let mut seq = 1;
        while result.is_none() && seq < 100 {
            let droplet = encoder.next_droplet();
            if seq % 3 != 0 {
                result = try_decode_compute_payload(&mut decoder, droplet).unwrap();
            }
            seq += 1;
        }

        let reconstructed = result.expect("payload successfully reconstructed despite drop");
        assert_eq!(reconstructed, original_tensor);
    }
}
