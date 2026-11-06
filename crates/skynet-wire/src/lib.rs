//! `skynet-wire`: Loss-tolerant `<= 1200-byte` Synaptic UDP wire protocol,
//! `Ed25519` datagram authentication, and ACK-less GF(2) Fountain Erasure Droplets.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use skynet_ledger::{SknTransaction, SynapticVoucher, TxOutput, UtreexoProof, UtxoId};
use thiserror::Error;

/// Strict IPv4/IPv6 fragmentation-free UDP datagram ceiling (`1200 bytes`).
pub const MAX_UDP_DATAGRAM_BYTES: usize = 1200;
/// Default SkyNet port (`8888`) for standalone UDP Agent nodes and telemetry/dashboard services.
pub const DEFAULT_PORT: u16 = 8888;
pub const WIRE_MAGIC: [u8; 2] = *b"SK";
pub const WIRE_VERSION: u8 = 1;
pub const MAX_ALLOWED_HOP_TTL: u8 = 16;

pub mod compute_wire;
pub mod fountain;
pub mod mcp_capsule;
pub mod opcode_registry;
pub mod peer_table;

pub use compute_wire::{
    encode_compute_payload, try_decode_compute_payload, ComputeAssignment, ComputeAuditorAttestation,
    ComputeAuditorBid, ComputeBid, ComputeCompletionClaim, ComputeMessage, ComputeTaskManifest,
    TaskId,
};
pub use fountain::{FountainDecoder, FountainDroplet, FountainEncoder, FOUNTAIN_BLOCK_BYTES};
pub use mcp_capsule::{CapsuleManifest, CapsuleTargetArch, HierarchicalOpcode, OpcodeDomain};
pub use opcode_registry::{
    lookup_opcode_descriptor, total_registered_capacity, DomainCategoryInfo, OpcodeDescriptor,
    DOMAIN_CATEGORIES,
};
pub use peer_table::{PeerContact, SynapticPeerTable};

#[derive(Debug, Error)]
pub enum WireError {
    #[error("UDP impulse exceeds 1200-byte MTU ceiling ({actual} bytes)")]
    OversizedDatagram { actual: usize },
    #[error("invalid magic bytes or unsupported wire version")]
    InvalidHeader,
    #[error("invalid hop TTL ({actual}): must be between 1 and {max}")]
    InvalidHopTtl { actual: u8, max: u8 },
    #[error("bincode serialization error: {0}")]
    Codec(#[from] bincode::Error),
    #[error("invalid Ed25519 packet signature")]
    BadSignature,
    #[error("fountain object exceeds maximum supported blocks")]
    FountainObjectTooLarge,
    #[error("fountain BLAKE3 digest mismatch after reconstruction")]
    FountainChecksumMismatch,
}

/// Payload variants carried inside a single `<= 1200B` UDP `ImpulsePacket`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpulseBody {
    /// Periodic biological heartbeat announcing Agent scale, health, and stake.
    Heartbeat {
        tier_code: u8,
        total_cells: u32,
        stem_cells: u32,
        on_chain_nano_skn: u128,
        elite_dna_hash: [u8; 32],
    },
    /// Unsolicited gossip of a signed SKN UTXO-DAG transaction.
    GossipTx(SknTransaction),
    /// Statelessly verifiable transaction carrying compact `Utreexo` Merkle witnesses (`<= 1200B`).
    WitnessedTx {
        tx: SknTransaction,
        spent_outputs: Vec<TxOutput>,
        proofs: Vec<UtreexoProof>,
    },
    /// Loss-tolerant cumulative `nanoSKN` micro-payment voucher + AI stimulus impulse.
    SynapticMicroPay {
        voucher: SynapticVoucher,
        stimulus: [f32; 8],
    },
    /// Loss-tolerant Snowball consensus poll querying contested UTXOs.
    SnowballPull {
        query_id: u64,
        contested_utxos: Vec<UtxoId>,
    },
    /// Compact Snowball vote reply (`(contested_utxo, preferred_tx_hash)`).
    SnowballReply {
        query_id: u64,
        votes: Vec<(UtxoId, [u8; 32])>,
    },
    /// Announces dynamic cluster prefix fission (`0bP -> 0bP0, 0bP1`).
    ClusterMitosis {
        parent_prefix: u32,
        parent_prefix_len: u8,
        epoch: u64,
    },
    /// Stateless Fountain erasure-coded droplet for syncing large `CellDna` or state snapshots.
    FountainDroplet(FountainDroplet),
    /// Distributed compute task announcement, bidding, data transfer, or completion claim.
    Compute(ComputeMessage),
}

/// Authenticated connectionless UDP packet (`<= 1200 bytes` on wire).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpulsePacket {
    pub magic: [u8; 2],
    pub version: u8,
    pub cluster_prefix: u32,
    pub prefix_len: u8,
    pub hop_ttl: u8,
    pub epoch: u64,
    pub pulse_seq: u32,
    pub sender_pubkey: [u8; 32],
    pub body: ImpulseBody,
    pub signature: Vec<u8>,
}

impl ImpulsePacket {
    pub fn new_signed(
        signing_key: &SigningKey,
        cluster_prefix: u32,
        prefix_len: u8,
        hop_ttl: u8,
        epoch: u64,
        pulse_seq: u32,
        body: ImpulseBody,
    ) -> Result<Self, WireError> {
        let sender_pubkey = signing_key.verifying_key().to_bytes();
        let mut pkt = Self {
            magic: WIRE_MAGIC,
            version: WIRE_VERSION,
            cluster_prefix,
            prefix_len,
            hop_ttl,
            epoch,
            pulse_seq,
            sender_pubkey,
            body,
            signature: Vec::new(),
        };
        let digest = pkt.signing_digest()?;
        let sig: Signature = signing_key.sign(&digest);
        pkt.signature = sig.to_bytes().to_vec();

        // Enforce strict 1200-byte MTU discipline at creation time
        let encoded_len = bincode::serialized_size(&pkt)? as usize;
        if encoded_len > MAX_UDP_DATAGRAM_BYTES {
            return Err(WireError::OversizedDatagram {
                actual: encoded_len,
            });
        }
        Ok(pkt)
    }

    pub fn signing_digest(&self) -> Result<[u8; 32], WireError> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SKYNET_UDP_IMPULSE_V1");
        hasher.update(&self.magic);
        hasher.update(&[self.version]);
        hasher.update(&self.cluster_prefix.to_le_bytes());
        hasher.update(&[self.prefix_len, self.hop_ttl]);
        hasher.update(&self.epoch.to_le_bytes());
        hasher.update(&self.pulse_seq.to_le_bytes());
        hasher.update(&self.sender_pubkey);
        let body_bytes = bincode::serialize(&self.body)?;
        hasher.update(&body_bytes);
        Ok(*hasher.finalize().as_bytes())
    }

    pub fn encode_datagram(&self) -> Result<Vec<u8>, WireError> {
        let bytes = bincode::serialize(self)?;
        if bytes.len() > MAX_UDP_DATAGRAM_BYTES {
            return Err(WireError::OversizedDatagram {
                actual: bytes.len(),
            });
        }
        Ok(bytes)
    }

    /// Fast pre-validation before CPU-heavy cryptographic verification
    /// to mitigate signature-flooding DoS attacks.
    pub fn decode_and_verify(raw: &[u8]) -> Result<Self, WireError> {
        if raw.len() > MAX_UDP_DATAGRAM_BYTES {
            return Err(WireError::OversizedDatagram { actual: raw.len() });
        }
        let pkt: Self = bincode::deserialize(raw)?;
        if pkt.magic != WIRE_MAGIC || pkt.version != WIRE_VERSION {
            return Err(WireError::InvalidHeader);
        }
        if pkt.hop_ttl == 0 || pkt.hop_ttl > MAX_ALLOWED_HOP_TTL {
            return Err(WireError::InvalidHopTtl {
                actual: pkt.hop_ttl,
                max: MAX_ALLOWED_HOP_TTL,
            });
        }

        let vk = VerifyingKey::from_bytes(&pkt.sender_pubkey)
            .map_err(|_| WireError::BadSignature)?;
        let sig_arr: [u8; 64] = pkt
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| WireError::BadSignature)?;
        let sig = Signature::from_bytes(&sig_arr);
        let digest = pkt.signing_digest()?;
        vk.verify(&digest, &sig)
            .map_err(|_| WireError::BadSignature)?;
        Ok(pkt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    use skynet_dna::CellDna;

    #[test]
    fn fountain_codes_reconstruct_cell_dna_over_40_percent_udp_packet_loss() {
        let dna = CellDna::genesis_stem(2026);
        let raw_dna = bincode::serialize(&dna).unwrap();
        assert!(raw_dna.len() > MAX_UDP_DATAGRAM_BYTES);

        let key = SigningKey::from_bytes(&[7u8; 32]);
        let mut enc = FountainEncoder::new(&raw_dna).unwrap();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        let mut decoder: Option<FountainDecoder> = None;
        let mut recovered: Option<Vec<u8>> = None;
        let mut sent = 0usize;
        let mut survived = 0usize;

        while recovered.is_none() && sent < 100 {
            let droplet = enc.next_droplet();
            let pkt = ImpulsePacket::new_signed(
                &key,
                0,
                0,
                4,
                1,
                sent as u32,
                ImpulseBody::FountainDroplet(droplet),
            )
            .unwrap();
            let wire_bytes = pkt.encode_datagram().unwrap();
            assert!(wire_bytes.len() <= MAX_UDP_DATAGRAM_BYTES);
            sent += 1;

            if rng.gen_bool(0.40) {
                continue;
            }
            survived += 1;
            let decoded_pkt = ImpulsePacket::decode_and_verify(&wire_bytes).unwrap();
            if let ImpulseBody::FountainDroplet(d) = decoded_pkt.body {
                let dec = decoder.get_or_insert_with(|| FountainDecoder::from_first_droplet(&d));
                if let Some(bytes) = dec.absorb(d).unwrap() {
                    recovered = Some(bytes);
                }
            }
        }

        assert!(recovered.is_some());
        assert!(survived < sent);
        let reconstructed_dna: CellDna = bincode::deserialize(&recovered.unwrap()).unwrap();
        assert_eq!(reconstructed_dna.compute_hash(), dna.compute_hash());
    }

    #[test]
    fn tampered_udp_datagram_is_rejected() {
        let key = SigningKey::from_bytes(&[9u8; 32]);
        let pkt = ImpulsePacket::new_signed(
            &key,
            0b10,
            2,
            3,
            10,
            1,
            ImpulseBody::Heartbeat {
                tier_code: 2,
                total_cells: 500,
                stem_cells: 40,
                on_chain_nano_skn: 1_000_000_000,
                elite_dna_hash: [1u8; 32],
            },
        )
        .unwrap();
        let mut bytes = pkt.encode_datagram().unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        assert!(ImpulsePacket::decode_and_verify(&bytes).is_err());
    }

    #[test]
    fn invalid_hop_ttl_rejected_before_crypto_eval() {
        let key = SigningKey::from_bytes(&[9u8; 32]);
        let pkt = ImpulsePacket::new_signed(
            &key,
            0b10,
            2,
            0, // invalid hop TTL
            10,
            1,
            ImpulseBody::Heartbeat {
                tier_code: 2,
                total_cells: 10,
                stem_cells: 2,
                on_chain_nano_skn: 100,
                elite_dna_hash: [0u8; 32],
            },
        )
        .unwrap();
        let bytes = pkt.encode_datagram().unwrap();
        let err = ImpulsePacket::decode_and_verify(&bytes).unwrap_err();
        matches!(err, WireError::InvalidHopTtl { .. });
    }

    #[test]
    fn synaptic_peer_table_bounds_10000_ips_under_6kb_ram_with_logarithmic_buckets() {
        let local_pub = [0xAAu8; 32];
        let mut table = SynapticPeerTable::new(&local_pub, 0b01, 2, 12, 2);
        let mut rng = ChaCha8Rng::seed_from_u64(9999);

        for i in 0..10_000u32 {
            if i % 10 == 0 {
                table.decay_synapses();
            }
            let mut pk = [0u8; 32];
            rng.fill(&mut pk);
            let port = (10_000 + (i % 50_000)) as u16;
            let addr: SocketAddr = format!("10.{}.{}.{}:{}", (i >> 16) & 255, (i >> 8) & 255, i & 255, port)
                .parse()
                .unwrap();
            table.observe_peer(PeerContact {
                pubkey: pk,
                addr,
                cluster_prefix: i % 4,
                prefix_len: 2,
                bonded_skn_whole: (i as u64 % 500) * 10,
                synaptic_score: 0,
            });
        }

        assert_eq!(table.total_observed_ips_ever, 10_000);
        assert!(table.stored_peer_count() <= 64);
        assert!(table.estimated_ram_bytes() < 6_000);
        assert!(table.total_hebbian_evictions > 1_000);
    }

    #[test]
    fn test_hebbian_resists_eclipse_decay_attack() {
        let local_pub = [0x55u8; 32];
        let mut table = SynapticPeerTable::new(&local_pub, 0, 0, 4, 1);

        let strong_pub = [0x11u8; 32];
        table.observe_peer(PeerContact {
            pubkey: strong_pub,
            addr: "192.168.1.10:8888".parse().unwrap(),
            cluster_prefix: 0,
            prefix_len: 0,
            bonded_skn_whole: 1000,
            synaptic_score: 500,
        });

        let initial_score = table.intra_cluster_peers[0].synaptic_score;

        for i in 0..50 {
            let mut fake_pk = [0u8; 32];
            fake_pk[0] = (i + 1) as u8;
            table.observe_peer(PeerContact {
                pubkey: fake_pk,
                addr: format!("10.0.0.{}:8888", i + 1).parse().unwrap(),
                cluster_prefix: 0,
                prefix_len: 0,
                bonded_skn_whole: 0,
                synaptic_score: 1,
            });
        }

        let incumbent = table.intra_cluster_peers.iter().find(|p| p.pubkey == strong_pub).unwrap();
        assert_eq!(incumbent.synaptic_score, initial_score);
    }
}
