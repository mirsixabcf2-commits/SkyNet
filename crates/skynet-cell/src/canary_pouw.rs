//! Proof-of-Useful-Work (PoUW) deterministic Canary Spot-Check (`O(1)`) generator
//! (`BLAKE3` XOF `"SKYNET_POUW_CANARY_V1"`) and dishonest-worker fraud verifier.

use serde::{Deserialize, Serialize};

use crate::compute_candle::compute_canonical_digest;
use crate::gradient_codec::{cosine_similarity, decode_dense_f16, encode_dense_f16, l2_norm};
use crate::micro_train::{MicroTrainBatch, MicroTrainConfig, MicroTrainResult};

pub const CANARY_XOF_DOMAIN: &[u8] = b"SKYNET_POUW_CANARY_V1";
const CANARY_ROLL_DOMAIN: &[u8] = b"SKYNET_POUW_CANARY_ROLL_V1";
pub const DEFAULT_CANARY_RATE_BPS: u16 = 1_000;
pub const DEFAULT_MIN_COSINE_MILLI: u16 = 985;
pub const DEFAULT_MAX_GRAD_NORM_MILLI: u32 = 5_000;
const DEFAULT_CANARY_SAMPLES: usize = 8;
const BPS_SCALE: u32 = 10_000;
const MILLI_SCALE: f32 = 1_000.0;
const NORM_F16_TOLERANCE: f32 = 1.02;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanaryConfig {
    pub canary_rate_bps: u16,
    pub min_cosine_milli: u16,
    pub max_grad_norm_milli: u32,
}

impl Default for CanaryConfig {
    fn default() -> Self {
        Self {
            canary_rate_bps: DEFAULT_CANARY_RATE_BPS,
            min_cosine_milli: DEFAULT_MIN_COSINE_MILLI,
            max_grad_norm_milli: DEFAULT_MAX_GRAD_NORM_MILLI,
        }
    }
}

impl CanaryConfig {
    /// Deterministic `10%` (`canary_rate_bps`) spot-check selector for a `(seed, epoch, shard_id)`.
    pub fn should_inject_canary(&self, seed: &[u8; 32], epoch: u64, shard_id: u16) -> bool {
        if self.canary_rate_bps == 0 {
            return false;
        }
        if self.canary_rate_bps >= BPS_SCALE as u16 {
            return true;
        }
        let mut hasher = blake3::Hasher::new();
        hasher.update(CANARY_ROLL_DOMAIN);
        hasher.update(seed);
        hasher.update(&epoch.to_le_bytes());
        hasher.update(&shard_id.to_le_bytes());
        let digest = hasher.finalize();
        let bytes = digest.as_bytes();
        let roll = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) % BPS_SCALE;
        roll < (self.canary_rate_bps as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FraudReason {
    NonFiniteValues,
    EmptyOrMismatchedDelta,
    DigestMismatch,
    LossDidNotDecrease,
    GradNormExceeded,
    ZeroNormDelta,
    CosineBelowThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryVerdict {
    Valid,
    FraudDetected {
        reason: FraudReason,
        measured_cosine_milli: i16,
    },
}

impl CanaryVerdict {
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

/// Synthesizes a deterministic Canary micro-batch via `BLAKE3` XOF (`"SKYNET_POUW_CANARY_V1"`).
pub fn generate_canary_batch(
    seed: &[u8; 32],
    epoch: u64,
    shard_id: u16,
    cfg: &MicroTrainConfig,
) -> MicroTrainBatch {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CANARY_XOF_DOMAIN);
    hasher.update(seed);
    hasher.update(&epoch.to_le_bytes());
    hasher.update(&shard_id.to_le_bytes());
    hasher.update(&(cfg.dim_in as u32).to_le_bytes());
    hasher.update(&(cfg.dim_hidden as u32).to_le_bytes());
    hasher.update(&(cfg.dim_out as u32).to_le_bytes());

    let mut xof = hasher.finalize_xof();
    let mut task_id = [0u8; 32];
    xof.fill(&mut task_id);

    let weights_f32 = sample_xof_range(&mut xof, cfg.expected_weights_len(), -0.25, 0.25);
    let samples_x = sample_xof_range(&mut xof, DEFAULT_CANARY_SAMPLES * cfg.dim_in, -0.8, 0.8);
    let targets_y = sample_xof_range(&mut xof, DEFAULT_CANARY_SAMPLES * cfg.dim_out, 0.45, 0.85);

    MicroTrainBatch {
        task_id,
        shard_id,
        base_weights_f16: encode_dense_f16(&weights_f32),
        samples_x,
        targets_y,
    }
}

/// Audits a Worker's `MicroTrainResult` against the deterministic reference Canary result.
pub fn verify_canary_spot_check(
    worker_res: &MicroTrainResult,
    reference_res: &MicroTrainResult,
    canary_cfg: &CanaryConfig,
) -> CanaryVerdict {
    if !worker_res.initial_loss.is_finite() || !worker_res.final_loss.is_finite() {
        return fraud(FraudReason::NonFiniteValues, 0);
    }
    let worker_delta = decode_dense_f16(&worker_res.delta_f16);
    let ref_delta = decode_dense_f16(&reference_res.delta_f16);
    if worker_delta.is_empty() || worker_delta.len() != ref_delta.len() {
        return fraud(FraudReason::EmptyOrMismatchedDelta, 0);
    }
    if worker_delta.iter().any(|v| !v.is_finite()) {
        return fraud(FraudReason::NonFiniteValues, 0);
    }
    if compute_canonical_digest(&worker_delta) != worker_res.canonical_digest {
        return fraud(FraudReason::DigestMismatch, 0);
    }
    if worker_res.final_loss > worker_res.initial_loss {
        return fraud(FraudReason::LossDidNotDecrease, 0);
    }
    let max_norm = ((canary_cfg.max_grad_norm_milli as f32) / MILLI_SCALE) * NORM_F16_TOLERANCE;
    if l2_norm(&worker_delta) > max_norm {
        return fraud(FraudReason::GradNormExceeded, 0);
    }
    let Some(cos) = cosine_similarity(&worker_delta, &ref_delta) else {
        return fraud(FraudReason::ZeroNormDelta, 0);
    };
    let cos_milli = (cos * MILLI_SCALE).round() as i16;
    if cos < (canary_cfg.min_cosine_milli as f32) / MILLI_SCALE {
        return fraud(FraudReason::CosineBelowThreshold, cos_milli);
    }
    CanaryVerdict::Valid
}

fn fraud(reason: FraudReason, measured_cosine_milli: i16) -> CanaryVerdict {
    CanaryVerdict::FraudDetected { reason, measured_cosine_milli }
}

fn sample_xof_range(xof: &mut blake3::OutputReader, count: usize, low: f32, high: f32) -> Vec<f32> {
    let span = high - low;
    (0..count)
        .map(|_| {
            let mut pair = [0u8; 2];
            xof.fill(&mut pair);
            let unit = (u16::from_le_bytes(pair) as f32) / (u16::MAX as f32);
            low + unit * span
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::micro_train::{execute_micro_train_slices, MicroTrainOutcome};
    use crate::preempt_guard::PreemptGuard;

    #[test]
    fn honest_canary_passes_and_byzantine_frauds_are_caught() {
        let cfg = MicroTrainConfig { dim_in: 4, dim_hidden: 6, dim_out: 2, inner_steps: 6, ..Default::default() };
        let batch = generate_canary_batch(&[7u8; 32], 1, 0, &cfg);
        let guard = PreemptGuard::new();
        let MicroTrainOutcome::Completed(ref_res) = execute_micro_train_slices(&cfg, &batch, None, &guard).expect("ref") else {
            panic!("expected completed reference");
        };
        assert!(ref_res.final_loss < ref_res.initial_loss);
        let canary_cfg = CanaryConfig::default();
        assert_eq!(verify_canary_spot_check(&ref_res, &ref_res, &canary_cfg), CanaryVerdict::Valid);

        let mut inverted = ref_res.clone();
        let neg: Vec<f32> = ref_res.decode_delta_f32().into_iter().map(|v| -v).collect();
        inverted.delta_f16 = encode_dense_f16(&neg);
        inverted.canonical_digest = compute_canonical_digest(&decode_dense_f16(&inverted.delta_f16));
        assert!(matches!(
            verify_canary_spot_check(&inverted, &ref_res, &canary_cfg),
            CanaryVerdict::FraudDetected { reason: FraudReason::CosineBelowThreshold, .. }
        ));

        let mut nan_claim = ref_res.clone();
        nan_claim.final_loss = f32::NAN;
        assert_eq!(
            verify_canary_spot_check(&nan_claim, &ref_res, &canary_cfg),
            CanaryVerdict::FraudDetected { reason: FraudReason::NonFiniteValues, measured_cosine_milli: 0 }
        );
    }
}
