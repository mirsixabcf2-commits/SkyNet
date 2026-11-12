//! `PlasmidPatch`: Quantized federated evolution patches (`SKYNET_PLASMID_PATCH_V1`)
//! and bounded guided adaptation onto `CellDna`.

use serde::{Deserialize, Serialize};

use crate::cell_dna::{CellDna, WEIGHT_CLAMP_LIMIT};
use crate::genes::{
    AdaptiveGenes, DnaError, MAX_BASIS_POINTS, MAX_LEAK_RATE, MAX_MUTATION_SIGMA, MIN_LEAK_RATE,
};

pub const PLASMID_PATCH_DOMAIN: &[u8] = b"SKYNET_PLASMID_PATCH_V1";
const PLASMID_ID_DOMAIN: &[u8] = b"SKYNET_PLASMID_PATCH_ID_V1";
pub const DEFAULT_PLASMID_SCALE_MILLI: u32 = 10_000_000;
const MILLI_PER_UNIT: f32 = 1_000.0;
const MAX_QUANTIZED_I16_F32: f32 = 30_000.0;
const MAX_SINGLE_PATCH_STEP: f32 = 1.5;
const MIN_PATCH_SIGMA_FLOOR: f32 = 0.005;

/// Quantized weight-delta plasmid emitted by federated micro-training aggregation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlasmidPatch {
    pub patch_id: [u8; 32],
    pub parent_dna_hash: [u8; 32],
    pub fitness_gain_bps: u16,
    pub quantized_deltas_i16: Vec<i16>,
    pub scale_milli: u32,
    pub canonical_digest: [u8; 32],
}

impl PlasmidPatch {
    /// Quantizes floating-point weight deltas into a signed/sealed `PlasmidPatch`.
    pub fn from_float_deltas(
        parent_dna_hash: [u8; 32],
        fitness_gain_bps: u16,
        deltas: &[f32],
    ) -> Result<Self, DnaError> {
        if deltas.iter().any(|d| !d.is_finite()) {
            return Err(DnaError::NonFiniteParameter);
        }
        let scale_milli = choose_scale_milli(deltas);
        let inv_step = (scale_milli as f32) / MILLI_PER_UNIT;
        let quantized_deltas_i16: Vec<i16> = deltas
            .iter()
            .map(|&d| (d * inv_step).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16)
            .collect();

        let patch_id = compute_patch_id(&parent_dna_hash, fitness_gain_bps, scale_milli, &quantized_deltas_i16);
        let mut patch = Self {
            patch_id,
            parent_dna_hash,
            fitness_gain_bps,
            quantized_deltas_i16,
            scale_milli,
            canonical_digest: [0u8; 32],
        };
        patch.canonical_digest = patch.compute_canonical_digest();
        Ok(patch)
    }

    /// Computes canonical BLAKE3 digest under domain `"SKYNET_PLASMID_PATCH_V1"`.
    pub fn compute_canonical_digest(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(PLASMID_PATCH_DOMAIN);
        hasher.update(&self.patch_id);
        hasher.update(&self.parent_dna_hash);
        hasher.update(&self.fitness_gain_bps.to_le_bytes());
        hasher.update(&self.scale_milli.to_le_bytes());
        let len = self.quantized_deltas_i16.len() as u32;
        hasher.update(&len.to_le_bytes());
        for &q in &self.quantized_deltas_i16 {
            hasher.update(&q.to_le_bytes());
        }
        *hasher.finalize().as_bytes()
    }

    /// Validates positive quantization scale and `"SKYNET_PLASMID_PATCH_V1"` digest.
    pub fn verify_digest(&self) -> Result<(), DnaError> {
        if self.scale_milli == 0 {
            return Err(DnaError::InvalidPlasmidScale);
        }
        if self.compute_canonical_digest() != self.canonical_digest {
            return Err(DnaError::PlasmidDigestMismatch);
        }
        Ok(())
    }

    /// Dequantizes `i16` deltas back into `f32` using `scale_milli`.
    pub fn dequantize_deltas(&self) -> Result<Vec<f32>, DnaError> {
        if self.scale_milli == 0 {
            return Err(DnaError::InvalidPlasmidScale);
        }
        let inv_step = (self.scale_milli as f32) / MILLI_PER_UNIT;
        Ok(self
            .quantized_deltas_i16
            .iter()
            .map(|&q| (q as f32) / inv_step)
            .collect())
    }
}

/// Applies a verified `PlasmidPatch` onto `dna`, returning a newly hashed child `CellDna`.
pub fn apply_plasmid_patch(dna: &CellDna, patch: &PlasmidPatch) -> Result<CellDna, DnaError> {
    dna.verify_integrity()?;
    patch.verify_digest()?;
    if patch.parent_dna_hash != dna.dna_id {
        return Err(DnaError::PlasmidParentMismatch);
    }

    let deltas = patch.dequantize_deltas()?;
    let fitness_ratio = (patch.fitness_gain_bps.min(MAX_BASIS_POINTS) as f32) / (MAX_BASIS_POINTS as f32);
    let blend_gain = (0.5 + 0.5 * fitness_ratio).clamp(0.5, 1.0);

    let mut child = dna.clone();
    apply_weight_deltas(&mut child, &deltas, blend_gain)?;
    adapt_genes_from_fitness(&mut child.adaptive, fitness_ratio);
    child.parent_dna_id = Some(dna.dna_id);
    child.generation = dna.generation.saturating_add(1);
    child.dna_id = child.compute_hash();
    child.verify_integrity()?;
    Ok(child)
}

fn apply_weight_deltas(child: &mut CellDna, deltas: &[f32], gain: f32) -> Result<(), DnaError> {
    let full_len = child.adaptive.expected_weight_len();
    let active_len = child.adaptive.active_operon_weight_len();
    if deltas.len() == full_len {
        for (w, &d) in child.weights.iter_mut().zip(deltas.iter()) {
            let step = (d * gain).clamp(-MAX_SINGLE_PATCH_STEP, MAX_SINGLE_PATCH_STEP);
            *w = (*w + step).clamp(-WEIGHT_CLAMP_LIMIT, WEIGHT_CLAMP_LIMIT);
        }
        return Ok(());
    }
    if deltas.len() != active_len {
        return Err(DnaError::WeightShapeMismatch {
            expected: full_len,
            actual: deltas.len(),
        });
    }
    let trunk_len = (child.adaptive.input_dim as usize) * (child.adaptive.hidden_dim as usize)
        + (child.adaptive.hidden_dim as usize);
    let head_len = active_len - trunk_len;
    let head_idx = child.epigenetics.expressed_role.operon_index().unwrap_or(0);
    let head_start = trunk_len + head_idx * head_len;
    for (w, &d) in child.weights[..trunk_len].iter_mut().zip(&deltas[..trunk_len]) {
        let step = (d * gain).clamp(-MAX_SINGLE_PATCH_STEP, MAX_SINGLE_PATCH_STEP);
        *w = (*w + step).clamp(-WEIGHT_CLAMP_LIMIT, WEIGHT_CLAMP_LIMIT);
    }
    for (w, &d) in child.weights[head_start..head_start + head_len]
        .iter_mut()
        .zip(&deltas[trunk_len..])
    {
        let step = (d * gain).clamp(-MAX_SINGLE_PATCH_STEP, MAX_SINGLE_PATCH_STEP);
        *w = (*w + step).clamp(-WEIGHT_CLAMP_LIMIT, WEIGHT_CLAMP_LIMIT);
    }
    Ok(())
}

fn adapt_genes_from_fitness(adaptive: &mut AdaptiveGenes, fitness_ratio: f32) {
    adaptive.mutation_sigma = (adaptive.mutation_sigma * (1.0 - 0.15 * fitness_ratio))
        .clamp(MIN_PATCH_SIGMA_FLOOR, MAX_MUTATION_SIGMA);
    adaptive.leak_rate =
        (adaptive.leak_rate + 0.02 * fitness_ratio).clamp(MIN_LEAK_RATE, MAX_LEAK_RATE);
}

fn choose_scale_milli(deltas: &[f32]) -> u32 {
    let max_abs = deltas.iter().fold(0.0f32, |acc, &d| acc.max(d.abs()));
    if max_abs <= 1e-6 {
        return DEFAULT_PLASMID_SCALE_MILLI;
    }
    let ideal_inv_step = (MAX_QUANTIZED_I16_F32 / max_abs).clamp(1.0, 10_000.0);
    (ideal_inv_step * MILLI_PER_UNIT).round() as u32
}

fn compute_patch_id(
    parent_dna_hash: &[u8; 32],
    fitness_gain_bps: u16,
    scale_milli: u32,
    quantized: &[i16],
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PLASMID_ID_DOMAIN);
    hasher.update(parent_dna_hash);
    hasher.update(&fitness_gain_bps.to_le_bytes());
    hasher.update(&scale_milli.to_le_bytes());
    for &q in quantized {
        hasher.update(&q.to_le_bytes());
    }
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genes::CellRole;

    #[test]
    fn plasmid_patch_roundtrip_and_guided_evolution_succeed() {
        let parent = CellDna::genesis_stem(21).differentiate(CellRole::Neural);
        let deltas = vec![0.05f32; parent.weights.len()];
        let patch = PlasmidPatch::from_float_deltas(parent.dna_id, 2_500, &deltas)
            .expect("valid plasmid patch");

        let child = apply_plasmid_patch(&parent, &patch).expect("apply patch succeeds");
        assert_eq!(child.parent_dna_id, Some(parent.dna_id));
        assert_eq!(child.generation, parent.generation + 1);
        assert!(child.verify_integrity().is_ok());
        assert_ne!(child.weights, parent.weights);
        assert!(child.adaptive.mutation_sigma < parent.adaptive.mutation_sigma);
    }

    #[test]
    fn plasmid_patch_rejects_tampered_digest_and_wrong_parent() {
        let parent = CellDna::genesis_stem(7);
        let active_len = parent.adaptive.active_operon_weight_len();
        let mut patch =
            PlasmidPatch::from_float_deltas(parent.dna_id, 1_000, &vec![-0.02; active_len])
                .expect("valid active-operon patch");
        assert!(apply_plasmid_patch(&parent, &patch).is_ok());

        let other_dna = CellDna::genesis_stem(8);
        assert_eq!(
            apply_plasmid_patch(&other_dna, &patch),
            Err(DnaError::PlasmidParentMismatch)
        );

        patch.quantized_deltas_i16[0] = patch.quantized_deltas_i16[0].saturating_add(1);
        assert_eq!(
            apply_plasmid_patch(&parent, &patch),
            Err(DnaError::PlasmidDigestMismatch)
        );
    }
}
