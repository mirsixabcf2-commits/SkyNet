//! `half::f16` dense & sparse Top-K weight-delta compression, L2 norm clipping,
//! and numerical cosine similarity utilities for federated micro-training.

use half::f16;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_SPARSE_DIMENSION: usize = (u16::MAX as usize) + 1;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum GradientCodecError {
    #[error("gradient or weight delta contains NaN or Inf")]
    NonFiniteGradient,
    #[error("max_norm must be finite and strictly positive")]
    MaxNormInvalid,
    #[error("vector length {len} exceeds u16 sparse index capacity")]
    DimensionExceedsSparseIndex { len: usize },
    #[error("sparse entry index {index} out of bounds for length {target_len}")]
    SparseIndexOutOfBounds { index: u16, target_len: usize },
}

/// Compact 4-byte `(index, f16_bits)` entry for sparse adapter / LoRA updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SparseTopKEntry {
    pub index: u16,
    pub f16_bits: u16,
}

/// Compresses `f32` weight deltas into raw IEEE-754 `half::f16` `u16` bit patterns.
pub fn encode_dense_f16(deltas: &[f32]) -> Vec<u16> {
    deltas.iter().map(|&v| f16::from_f32(v).to_bits()).collect()
}

/// Validated `f16` compression rejecting any `NaN`/`Inf` input or `f16` overflow.
pub fn encode_dense_f16_checked(deltas: &[f32]) -> Result<Vec<u16>, GradientCodecError> {
    if deltas.iter().any(|v| !v.is_finite()) {
        return Err(GradientCodecError::NonFiniteGradient);
    }
    let encoded = encode_dense_f16(deltas);
    if encoded.iter().any(|&b| !f16::from_bits(b).is_finite()) {
        return Err(GradientCodecError::NonFiniteGradient);
    }
    Ok(encoded)
}

/// Decompresses `half::f16` `u16` bit patterns back into `f32` deltas.
pub fn decode_dense_f16(bits: &[u16]) -> Vec<f32> {
    bits.iter().map(|&b| f16::from_bits(b).to_f32()).collect()
}

/// Selects top-`k` largest-magnitude entries and encodes them as sorted `SparseTopKEntry`.
pub fn encode_sparse_topk(
    deltas: &[f32],
    k: usize,
) -> Result<Vec<SparseTopKEntry>, GradientCodecError> {
    if deltas.len() > MAX_SPARSE_DIMENSION {
        return Err(GradientCodecError::DimensionExceedsSparseIndex { len: deltas.len() });
    }
    if deltas.iter().any(|v| !v.is_finite()) {
        return Err(GradientCodecError::NonFiniteGradient);
    }
    if k == 0 || deltas.is_empty() {
        return Ok(Vec::new());
    }
    let mut indexed: Vec<(usize, f32)> = deltas.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()).then_with(|| a.0.cmp(&b.0)));
    indexed.truncate(k.min(deltas.len()));
    indexed.sort_by_key(|&(idx, _)| idx);

    let mut out = Vec::with_capacity(indexed.len());
    for (idx, val) in indexed {
        let f16_val = f16::from_f32(val);
        if !f16_val.is_finite() {
            return Err(GradientCodecError::NonFiniteGradient);
        }
        out.push(SparseTopKEntry { index: idx as u16, f16_bits: f16_val.to_bits() });
    }
    Ok(out)
}

/// Reconstructs a dense `f32` vector of length `target_len` from `SparseTopKEntry` items.
pub fn decode_sparse_topk(
    entries: &[SparseTopKEntry],
    target_len: usize,
) -> Result<Vec<f32>, GradientCodecError> {
    let mut dense = vec![0.0f32; target_len];
    for entry in entries {
        let idx = entry.index as usize;
        if idx >= target_len {
            return Err(GradientCodecError::SparseIndexOutOfBounds {
                index: entry.index,
                target_len,
            });
        }
        let val = f16::from_bits(entry.f16_bits).to_f32();
        if !val.is_finite() {
            return Err(GradientCodecError::NonFiniteGradient);
        }
        dense[idx] = val;
    }
    Ok(dense)
}

/// Computes `f64`-accumulated Euclidean L2 norm of `values`.
pub fn l2_norm(values: &[f32]) -> f32 {
    values
        .iter()
        .map(|&v| {
            let d = v as f64;
            d * d
        })
        .sum::<f64>()
        .sqrt() as f32
}

/// Clips `values` in-place so `||values||_2 <= max_norm`, returning the pre-clip L2 norm.
pub fn clip_l2_norm(values: &mut [f32], max_norm: f32) -> Result<f32, GradientCodecError> {
    if !max_norm.is_finite() || max_norm <= 0.0 {
        return Err(GradientCodecError::MaxNormInvalid);
    }
    if values.iter().any(|v| !v.is_finite()) {
        return Err(GradientCodecError::NonFiniteGradient);
    }
    let norm = l2_norm(values);
    if norm > max_norm && norm > 0.0 {
        let scale = max_norm / norm;
        for v in values.iter_mut() {
            *v *= scale;
        }
    }
    Ok(norm)
}

/// Computes cosine similarity in `[-1.0, 1.0]`, returning `None` on zero-norm or invalid inputs.
pub fn cosine_similarity(lhs: &[f32], rhs: &[f32]) -> Option<f32> {
    if lhs.is_empty() || lhs.len() != rhs.len() {
        return None;
    }
    if lhs.iter().any(|v| !v.is_finite()) || rhs.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let mut dot = 0.0f64;
    let mut norm_l = 0.0f64;
    let mut norm_r = 0.0f64;
    for (&a, &b) in lhs.iter().zip(rhs.iter()) {
        let (da, db) = (a as f64, b as f64);
        dot += da * db;
        norm_l += da * da;
        norm_r += db * db;
    }
    let denom = (norm_l * norm_r).sqrt();
    if denom <= f64::EPSILON {
        return None;
    }
    Some((dot / denom).clamp(-1.0, 1.0) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_and_sparse_f16_roundtrip_preserve_dominant_components() {
        let src = vec![0.01f32, -0.75, 0.002, 1.25, -0.5];
        let decoded = decode_dense_f16(&encode_dense_f16(&src));
        for (a, b) in src.iter().zip(decoded.iter()) {
            assert!((a - b).abs() < 1e-3);
        }
        let sparse = encode_sparse_topk(&src, 2).expect("top-2");
        assert_eq!(sparse.len(), 2);
        let dense = decode_sparse_topk(&sparse, src.len()).expect("decode top-2");
        assert!((dense[1] - (-0.75)).abs() < 1e-3);
        assert!((dense[3] - 1.25).abs() < 1e-3);
        assert_eq!(dense[0], 0.0);
    }

    #[test]
    fn clip_l2_norm_bounds_magnitude_and_rejects_nan() {
        let mut grad = vec![3.0f32, 4.0f32];
        let orig = clip_l2_norm(&mut grad, 2.5).expect("clip succeeds");
        assert!((orig - 5.0).abs() < 1e-5);
        assert!((l2_norm(&grad) - 2.5).abs() < 1e-5);
        assert!((cosine_similarity(&grad, &[6.0, 8.0]).expect("cos") - 1.0).abs() < 1e-5);

        let mut bad = vec![1.0f32, f32::NAN];
        assert_eq!(clip_l2_norm(&mut bad, 1.0), Err(GradientCodecError::NonFiniteGradient));
    }
}
