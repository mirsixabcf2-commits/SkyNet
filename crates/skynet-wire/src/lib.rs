//! `skynet-wire`: Loss-tolerant `<= 1200-byte` Synaptic UDP wire protocol,
//! `< 5µs` stack-based `EpidermalFilter`, `Ed25519` datagram authentication,
//! P2P NAT rendezvous, and ACK-less GF(2) Fountain Erasure Droplets.

pub mod compute_wire;
pub mod epidermal_filter;
pub mod fountain;
pub mod mcp_capsule;
pub mod micro_train_wire;
pub mod opcode_domains;
pub mod opcode_registry;
pub mod packet;
pub mod peer_table;
pub mod rendezvous_wire;

#[cfg(test)]
mod mcp_capsule_tests;
#[cfg(test)]
mod wire_tests;

pub use compute_wire::{
    encode_compute_payload, try_decode_compute_payload, ComputeAssignment,
    ComputeAuditorAttestation, ComputeAuditorBid, ComputeBid, ComputeCompletionClaim,
    ComputeMessage, ComputeTaskManifest, TaskId,
};
pub use epidermal_filter::{
    EpidermalFilter, EpidermalHeader, EPIDERMAL_HEADER_BYTES, MAX_TIMESTAMP_DRIFT_SEC,
};
pub use fountain::{FountainDecoder, FountainDroplet, FountainEncoder, FOUNTAIN_BLOCK_BYTES};
pub use mcp_capsule::{CapsuleManifest, CapsuleTargetArch, HierarchicalOpcode, OpcodeDomain};
pub use micro_train_wire::{
    encode_micro_train_shard, try_decode_micro_train_shard, CanaryFraudReport,
    ChunkVoucherDelivery, MicroBatchManifest, MicroTrainWireMessage, WeightDeltaClaim,
    MAX_INLINE_DELTA_F16_LEN, MAX_MICRO_TRAIN_SHARD_BYTES,
};
pub use opcode_registry::{
    lookup_opcode_descriptor, total_registered_capacity, DomainCategoryInfo, OpcodeDescriptor,
    DOMAIN_CATEGORIES,
};
pub use packet::{
    ImpulseBody, ImpulsePacket, ImpulseSignOptions, WireError, DEFAULT_PORT, MAX_ALLOWED_HOP_TTL,
    MAX_UDP_DATAGRAM_BYTES, WIRE_MAGIC, WIRE_VERSION,
};
pub use peer_table::{PeerContact, SynapticPeerTable};
pub use rendezvous_wire::{
    GangliaRelayFrame, LanBeacon, PheromoneHello, PheromoneHelloParams, PunchProbe,
    PunchRendezvous, PunchRendezvousParams, RendezvousMessage, MAX_GANGLIA_RELAY_INNER_BYTES,
    MAX_RENDEZVOUS_CONTROL_BYTES,
};
