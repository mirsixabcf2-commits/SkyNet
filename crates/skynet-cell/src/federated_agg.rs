//! Byzantine-resilient `TrimmedMeanAggregator`: per-submission L2 clipping,
//! coordinate-wise `10%` tail trimming (`>= 4` samples), reputation weighting
//! (`pouw_score_bps`), and `PlasmidPatch` synthesis.

use serde::{Deserialize, Serialize};
use skynet_dna::{DnaError, PlasmidPatch};
use thiserror::Error;

use crate::compute_candle::compute_canonical_digest;
use crate::gradient_codec::{clip_l2_norm, decode_dense_f16, encode_dense_f16};

pub const DEFAULT_TRIM_RATIO_BPS: u16 = 1_000;
pub const MIN_SAMPLES_FOR_TRIM: usize = 4;
pub const DEFAULT_AGG_MAX_NORM_MILLI: u32 = 2_000;
const MAX_ALLOWED_TRIM_BPS: u16 = 4_999;
const BPS_DENOMINATOR: usize = 10_000;
const MILLI_SCALE: f32 = 1_000.0;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AggregationError {
    #[error("invalid trim_ratio_bps {0}: must be < 5000")]
    InvalidTrimRatio(u16),
    #[error("max_grad_norm_milli must be strictly positive")]
    InvalidMaxNorm,
    #[error("no valid non-finite / positive-reputation submissions to aggregate")]
    NoValidSubmissions,
    #[error("submission delta dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("DNA plasmid packaging error: {0}")]
    Dna(#[from] DnaError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeightedDeltaSubmission {
    pub worker_id: [u8; 32],
    pub delta_f16: Vec<u16>,
    pub pouw_score_bps: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregatedDelta {
    pub delta_f32: Vec<f32>,
    pub delta_f16: Vec<u16>,
    pub accepted_submissions: usize,
    pub rejected_submissions: usize,
    pub canonical_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrimmedMeanAggregator {
    pub trim_ratio_bps: u16,
    pub max_grad_norm_milli: u32,
}

impl Default for TrimmedMeanAggregator {
    fn default() -> Self {
        Self {
            trim_ratio_bps: DEFAULT_TRIM_RATIO_BPS,
            max_grad_norm_milli: DEFAULT_AGG_MAX_NORM_MILLI,
        }
    }
}

struct SanitizedSubmissions {
    samples: Vec<(Vec<f32>, u16)>,
    rejected: usize,
    dim: usize,
}

impl TrimmedMeanAggregator {
    pub fn new(trim_ratio_bps: u16, max_grad_norm_milli: u32) -> Result<Self, AggregationError> {
        if trim_ratio_bps > MAX_ALLOWED_TRIM_BPS {
            return Err(AggregationError::InvalidTrimRatio(trim_ratio_bps));
        }
        if max_grad_norm_milli == 0 {
            return Err(AggregationError::InvalidMaxNorm);
        }
        Ok(Self { trim_ratio_bps, max_grad_norm_milli })
    }

    /// Aggregates worker deltas by L2-clipping each vector, trimming top/bottom tails
    /// per coordinate when `>= 4` samples exist, and weighting by `pouw_score_bps`.
    pub fn aggregate(
        &self,
        submissions: &[WeightedDeltaSubmission],
    ) -> Result<AggregatedDelta, AggregationError> {
        if self.trim_ratio_bps > MAX_ALLOWED_TRIM_BPS {
            return Err(AggregationError::InvalidTrimRatio(self.trim_ratio_bps));
        }
        if self.max_grad_norm_milli == 0 {
            return Err(AggregationError::InvalidMaxNorm);
        }
        let max_norm = (self.max_grad_norm_milli as f32) / MILLI_SCALE;
        let clean = sanitize_submissions(submissions, max_norm)?;
        let n = clean.samples.len();
        let trim_k = compute_trim_count(n, self.trim_ratio_bps);

        let mut raw_out = Vec::with_capacity(clean.dim);
        let mut coord_buf: Vec<(f32, u16)> = Vec::with_capacity(n);
        for c in 0..clean.dim {
            coord_buf.clear();
            for (vec, score) in &clean.samples {
                coord_buf.push((vec[c], *score));
            }
            coord_buf.sort_by(|a, b| a.0.total_cmp(&b.0));
            raw_out.push(weighted_slice_mean(&coord_buf[trim_k..(n - trim_k)]));
        }

        let delta_f16 = encode_dense_f16(&raw_out);
        let delta_f32 = decode_dense_f16(&delta_f16);
        let canonical_digest = compute_canonical_digest(&delta_f32);
        Ok(AggregatedDelta {
            delta_f32,
            delta_f16,
            accepted_submissions: n,
            rejected_submissions: clean.rejected,
            canonical_digest,
        })
    }

    /// Aggregates worker deltas and seals the result into an evolution `PlasmidPatch`.
    pub fn aggregate_to_plasmid_patch(
        &self,
        submissions: &[WeightedDeltaSubmission],
        parent_dna_hash: [u8; 32],
        fitness_gain_bps: u16,
    ) -> Result<(AggregatedDelta, PlasmidPatch), AggregationError> {
        let aggregated = self.aggregate(submissions)?;
        let patch = PlasmidPatch::from_float_deltas(
            parent_dna_hash,
            fitness_gain_bps,
            &aggregated.delta_f32,
        )?;
        Ok((aggregated, patch))
    }
}

fn sanitize_submissions(
    submissions: &[WeightedDeltaSubmission],
    max_norm: f32,
) -> Result<SanitizedSubmissions, AggregationError> {
    let mut samples = Vec::with_capacity(submissions.len());
    let mut rejected = 0usize;
    let mut expected_dim: Option<usize> = None;

    for sub in submissions {
        if sub.pouw_score_bps == 0 || sub.delta_f16.is_empty() {
            rejected += 1;
            continue;
        }
        let dim = *expected_dim.get_or_insert(sub.delta_f16.len());
        if sub.delta_f16.len() != dim {
            return Err(AggregationError::DimensionMismatch { expected: dim, actual: sub.delta_f16.len() });
        }
        let mut decoded = decode_dense_f16(&sub.delta_f16);
        if clip_l2_norm(&mut decoded, max_norm).is_err() {
            rejected += 1;
            continue;
        }
        samples.push((decoded, sub.pouw_score_bps));
    }
    let Some(dim) = expected_dim.filter(|_| !samples.is_empty()) else {
        return Err(AggregationError::NoValidSubmissions);
    };
    Ok(SanitizedSubmissions { samples, rejected, dim })
}

fn compute_trim_count(n: usize, trim_ratio_bps: u16) -> usize {
    if n < MIN_SAMPLES_FOR_TRIM || trim_ratio_bps == 0 {
        return 0;
    }
    ((n * (trim_ratio_bps as usize)) / BPS_DENOMINATOR).max(1).min((n - 1) / 2)
}

fn weighted_slice_mean(items: &[(f32, u16)]) -> f32 {
    let mut weighted_sum = 0.0f64;
    let mut total_weight = 0.0f64;
    for &(val, score_bps) in items {
        let w = score_bps as f64;
        weighted_sum += (val as f64) * w;
        total_weight += w;
    }
    if total_weight <= 0.0 {
        return 0.0;
    }
    (weighted_sum / total_weight) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gradient_codec::cosine_similarity;
    use skynet_dna::{apply_plasmid_patch, CellDna, CellRole};

    #[test]
    fn trimmed_mean_converges_under_30_percent_byzantine_poisoning() {
        let dna = CellDna::genesis_stem(123).differentiate(CellRole::Neural);
        let dim = dna.adaptive.active_operon_weight_len();
        let honest_vec: Vec<f32> = (0..dim).map(|i| if i % 2 == 0 { -0.08 } else { 0.06 }).collect();

        let mut subs = Vec::new();
        for id in 0..7u8 {
            let jitter = (id as f32 - 3.0) * 0.001;
            let v: Vec<f32> = honest_vec.iter().map(|&x| x + jitter).collect();
            subs.push(WeightedDeltaSubmission { worker_id: [id; 32], delta_f16: encode_dense_f16(&v), pouw_score_bps: 8_000 });
        }
        subs.push(WeightedDeltaSubmission { worker_id: [91; 32], delta_f16: encode_dense_f16(&vec![500.0; dim]), pouw_score_bps: 3_000 });
        subs.push(WeightedDeltaSubmission { worker_id: [92; 32], delta_f16: encode_dense_f16(&vec![-500.0; dim]), pouw_score_bps: 3_000 });
        subs.push(WeightedDeltaSubmission { worker_id: [93; 32], delta_f16: vec![half::f16::NAN.to_bits(); dim], pouw_score_bps: 5_000 });

        let agg = TrimmedMeanAggregator::default();
        let (out, patch) = agg.aggregate_to_plasmid_patch(&subs, dna.dna_id, 2_000).expect("agg");
        assert_eq!(out.accepted_submissions, 9);
        assert_eq!(out.rejected_submissions, 1);
        assert!(cosine_similarity(&out.delta_f32, &honest_vec).expect("cos") > 0.99);
        assert!(apply_plasmid_patch(&dna, &patch).is_ok());
    }
}
