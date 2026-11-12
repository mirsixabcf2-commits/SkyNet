//! `micro_train_wire`: Opportunistic edge micro-training manifests, 16-bit weight delta
//! claims, PoUW canary fraud reports, per-chunk voucher delivery (`<= 1200B`), and `<= 128 KB`
//! Fountain erasure-coded shard transport helpers.

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use skynet_ledger::SynapticVoucher;

use crate::compute_wire::TaskId;
use crate::fountain::{FountainDecoder, FountainDroplet, FountainEncoder};
use crate::packet::{sign_ed25519_digest, verify_ed25519_sig, WireError};

/// Maximum payload size (`128 KB` = `256` blocks of `512 B`) per micro-training shard.
pub const MAX_MICRO_TRAIN_SHARD_BYTES: usize = 128 * 1024;
/// Maximum number of inline `f16` delta elements carried inside a single `<= 1200B` claim packet.
pub const MAX_INLINE_DELTA_F16_LEN: usize = 320;

/// Broadcast or assigned micro-batch training shard specification (`<= 1200B`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroBatchManifest {
    pub task_id: TaskId,
    pub client_pubkey: [u8; 32],
    pub opcode: u16,
    pub shard_id: u16,
    pub total_shards: u16,
    pub dim_in: u16,
    pub dim_hidden: u16,
    pub dim_out: u16,
    pub inner_steps: u16,
    pub learning_rate_micro: u32,
    pub shard_object_hash: [u8; 32],
    pub shard_bytes_len: u32,
    pub bounty_per_chunk_nano: u128,
    pub min_worker_bond_nano: u128,
}

/// Worker claim submitting quantized 16-bit weight deltas and loss convergence metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeightDeltaClaim {
    pub task_id: TaskId,
    pub shard_id: u16,
    pub worker_pubkey: [u8; 32],
    pub completed_steps: u16,
    pub initial_loss_micro: u64,
    pub final_loss_micro: u64,
    pub canonical_digest: [u8; 32],
    pub inline_delta_f16: Vec<u16>,
    pub delta_object_hash: Option<[u8; 32]>,
    pub worker_signature: Vec<u8>,
}

impl WeightDeltaClaim {
    pub fn into_signed(mut self, worker_key: &SigningKey) -> Result<Self, WireError> {
        if self.inline_delta_f16.len() > MAX_INLINE_DELTA_F16_LEN {
            return Err(WireError::OversizedDatagram { actual: self.inline_delta_f16.len() * 2 });
        }
        self.worker_pubkey = worker_key.verifying_key().to_bytes();
        self.worker_signature.clear();
        self.worker_signature = sign_ed25519_digest(worker_key, &self.signing_digest()?);
        Ok(self)
    }

    pub fn signing_digest(&self) -> Result<[u8; 32], WireError> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SKYNET_WEIGHT_DELTA_CLAIM_V1");
        let fields = (
            &self.task_id,
            self.shard_id,
            &self.worker_pubkey,
            self.completed_steps,
            self.initial_loss_micro,
            self.final_loss_micro,
            &self.canonical_digest,
            &self.inline_delta_f16,
            &self.delta_object_hash,
        );
        hasher.update(&bincode::serialize(&fields)?);
        Ok(*hasher.finalize().as_bytes())
    }

    pub fn verify_signature(&self) -> Result<(), WireError> {
        verify_ed25519_sig(&self.worker_pubkey, &self.signing_digest()?, &self.worker_signature)
    }
}

/// Signed PoUW Canary spot-check fraud report triggering worker bond slashing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryFraudReport {
    pub task_id: TaskId,
    pub shard_id: u16,
    pub faulty_worker: [u8; 32],
    pub challenger_auditor: [u8; 32],
    pub measured_cosine_milli: i16,
    pub evidence_digest: [u8; 32],
    pub auditor_signature: Vec<u8>,
}

impl CanaryFraudReport {
    pub fn into_signed(mut self, auditor_key: &SigningKey) -> Self {
        self.challenger_auditor = auditor_key.verifying_key().to_bytes();
        self.auditor_signature.clear();
        self.auditor_signature = sign_ed25519_digest(auditor_key, &self.signing_digest());
        self
    }

    pub fn signing_digest(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SKYNET_CANARY_FRAUD_REPORT_V1");
        hasher.update(&self.task_id);
        hasher.update(&self.shard_id.to_le_bytes());
        hasher.update(&self.faulty_worker);
        hasher.update(&self.challenger_auditor);
        hasher.update(&self.measured_cosine_milli.to_le_bytes());
        hasher.update(&self.evidence_digest);
        *hasher.finalize().as_bytes()
    }

    pub fn verify_signature(&self) -> Result<(), WireError> {
        verify_ed25519_sig(&self.challenger_auditor, &self.signing_digest(), &self.auditor_signature)
    }
}

/// Loss-tolerant per-chunk cumulative `SynapticVoucher` streaming settlement envelope (`<= 1200B`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkVoucherDelivery {
    pub task_id: TaskId,
    pub chunk_index: u16,
    pub total_chunks: u16,
    pub voucher: SynapticVoucher,
}

/// Wire variants for opportunistic micro-training transported inside `ImpulseBody::MicroTrain`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MicroTrainWireMessage {
    MicroBatchManifest(MicroBatchManifest),
    WeightDeltaClaim(WeightDeltaClaim),
    CanaryFraudReport(CanaryFraudReport),
    ChunkVoucherDelivery(ChunkVoucherDelivery),
}

/// Encodes a micro-training data or weight shard (`<= 128 KB`) into GF(2) Fountain droplets.
pub fn encode_micro_train_shard(shard_bytes: &[u8]) -> Result<FountainEncoder, WireError> {
    if shard_bytes.len() > MAX_MICRO_TRAIN_SHARD_BYTES {
        return Err(WireError::OversizedShard {
            actual: shard_bytes.len(),
            max: MAX_MICRO_TRAIN_SHARD_BYTES,
        });
    }
    FountainEncoder::new(shard_bytes)
}

/// Absorbs one Fountain droplet and returns the reconstructed `<= 128 KB` shard upon full rank.
pub fn try_decode_micro_train_shard(
    decoder: &mut FountainDecoder,
    droplet: FountainDroplet,
) -> Result<Option<Vec<u8>>, WireError> {
    let Some(bytes) = decoder.absorb(droplet)? else {
        return Ok(None);
    };
    if bytes.len() > MAX_MICRO_TRAIN_SHARD_BYTES {
        return Err(WireError::OversizedShard {
            actual: bytes.len(),
            max: MAX_MICRO_TRAIN_SHARD_BYTES,
        });
    }
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::{ImpulseBody, ImpulsePacket, MAX_UDP_DATAGRAM_BYTES};
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaCha8Rng;
    use skynet_ledger::{NanoSkn, UtxoId};

    #[test]
    fn test_delta_claim_fraud_and_voucher_fit_1200b_mtu() {
        let key = SigningKey::from_bytes(&[55u8; 32]);
        let claim = WeightDeltaClaim {
            task_id: [1u8; 32], shard_id: 2, worker_pubkey: [0u8; 32], completed_steps: 8,
            initial_loss_micro: 950_000, final_loss_micro: 410_000, canonical_digest: [9u8; 32],
            inline_delta_f16: vec![0x3C00; 256], delta_object_hash: None, worker_signature: vec![],
        }
        .into_signed(&key)
        .unwrap();
        claim.verify_signature().unwrap();
        let c_pkt = ImpulsePacket::new_signed(&key, 0, 0, 4, 1, 1, ImpulseBody::MicroTrain(MicroTrainWireMessage::WeightDeltaClaim(claim))).unwrap();
        assert!(c_pkt.encode_datagram().unwrap().len() <= MAX_UDP_DATAGRAM_BYTES);

        let fraud = CanaryFraudReport {
            task_id: [1u8; 32], shard_id: 2, faulty_worker: [8u8; 32], challenger_auditor: [0u8; 32],
            measured_cosine_milli: 410, evidence_digest: [7u8; 32], auditor_signature: vec![],
        }
        .into_signed(&key);
        fraud.verify_signature().unwrap();

        let voucher = SynapticVoucher::new_signed(&key, UtxoId { tx_hash: [2u8; 32], vout: 0 }, [3u8; 32], 1, NanoSkn(50_000));
        let v_msg = MicroTrainWireMessage::ChunkVoucherDelivery(ChunkVoucherDelivery { task_id: [1u8; 32], chunk_index: 1, total_chunks: 10, voucher });
        let v_pkt = ImpulsePacket::new_signed(&key, 0, 0, 4, 1, 2, ImpulseBody::MicroTrain(v_msg)).unwrap();
        assert!(v_pkt.encode_datagram().unwrap().len() <= MAX_UDP_DATAGRAM_BYTES);
    }

    #[test]
    fn test_fountain_128kb_shard_recovers_100_pct_under_40_pct_loss() {
        assert!(encode_micro_train_shard(&vec![0u8; MAX_MICRO_TRAIN_SHARD_BYTES + 1]).is_err());
        let mut shard = vec![0u8; MAX_MICRO_TRAIN_SHARD_BYTES];
        let mut rng = ChaCha8Rng::seed_from_u64(20261112);
        rng.fill(&mut shard[..]);

        let key = SigningKey::from_bytes(&[77u8; 32]);
        let mut enc = encode_micro_train_shard(&shard).unwrap();
        let mut dec: Option<FountainDecoder> = None;
        let mut recovered: Option<Vec<u8>> = None;

        for seq in 0..700u32 {
            let pkt = ImpulsePacket::new_signed(&key, 0, 0, 4, 1, seq, ImpulseBody::FountainDroplet(enc.next_droplet())).unwrap();
            let wire = pkt.encode_datagram().unwrap();
            assert!(wire.len() <= MAX_UDP_DATAGRAM_BYTES);
            if rng.gen_bool(0.40) {
                continue;
            }
            let ImpulseBody::FountainDroplet(d) = ImpulsePacket::decode_and_verify(&wire).unwrap().body else {
                continue;
            };
            let d_ref = dec.get_or_insert_with(|| FountainDecoder::from_first_droplet(&d));
            recovered = try_decode_micro_train_shard(d_ref, d).unwrap();
            if recovered.is_some() {
                break;
            }
        }
        assert_eq!(recovered.as_deref(), Some(shard.as_slice()));
    }
}
