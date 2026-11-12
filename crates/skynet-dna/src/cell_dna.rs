//! `CellDna`: Heritable genotype, BLAKE3 integrity verification, epigenetic
//! differentiation, `1 -> 2` mitosis replication, and horizontal crossover.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::genes::{
    AdaptiveGenes, CellRole, ConservedGenes, DnaError, EpigeneticMask, MAX_FEC_REDUNDANCY_BPS,
    MAX_GOSSIP_FANOUT, MAX_LEAK_RATE, MAX_PULSE_INTERVAL_MS, MIN_FEC_REDUNDANCY_BPS,
    MIN_GOSSIP_FANOUT, MIN_LEAK_RATE, MIN_PULSE_INTERVAL_MS,
};

pub const WEIGHT_CLAMP_LIMIT: f32 = 4.0;
const DNA_HASH_DOMAIN: &[u8] = b"SKYNET_DNA_V1";
const MITOSIS_WEIGHT_MUTATION_PROB: f64 = 0.35;
const MITOSIS_TRAIT_MUTATION_PROB: f64 = 0.25;
const MITOSIS_SIGMA_MIN: f32 = 0.005;
const MITOSIS_SIGMA_MAX: f32 = 0.15;
const RECIPIENT_BLEND_RATIO: f32 = 0.7;
const DONOR_BLEND_RATIO: f32 = 0.3;
const CROSSOVER_WEIGHT_PROB: f64 = 0.30;

/// Complete heritable DNA carried by every `MicroCell`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellDna {
    pub dna_id: [u8; 32],
    pub parent_dna_id: Option<[u8; 32]>,
    pub generation: u64,
    pub conserved: ConservedGenes,
    pub adaptive: AdaptiveGenes,
    pub epigenetics: EpigeneticMask,
    /// Flattened neural parameters: shared sensory trunk + 4 specialized operon heads.
    pub weights: Vec<f32>,
}

impl CellDna {
    /// Synthesizes a Generation-0 pluripotent Stem Cell DNA from a deterministic seed.
    pub fn genesis_stem(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let adaptive = AdaptiveGenes::default();
        let len = adaptive.expected_weight_len();
        let scale = (2.0 / (adaptive.input_dim as f32 + adaptive.hidden_dim as f32)).sqrt();
        let weights: Vec<f32> = (0..len).map(|_| rng.gen_range(-scale..scale)).collect();
        let mut dna = Self {
            dna_id: [0u8; 32],
            parent_dna_id: None,
            generation: 0,
            conserved: ConservedGenes::default(),
            adaptive,
            epigenetics: EpigeneticMask::stem(),
            weights,
        };
        dna.dna_id = dna.compute_hash();
        dna
    }

    /// Computes a deterministic BLAKE3 digest over the entire genotype & epigenome.
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(DNA_HASH_DOMAIN);
        match &self.parent_dna_id {
            Some(parent) => {
                hasher.update(&[1u8]);
                hasher.update(parent);
            }
            None => {
                hasher.update(&[0u8]);
            }
        }
        hasher.update(&self.generation.to_le_bytes());
        update_genes_digest(&mut hasher, &self.conserved, &self.adaptive);
        hasher.update(&[self.epigenetics.expressed_role.role_tag()]);
        for g in &self.epigenetics.operon_gains {
            hasher.update(&g.to_bits().to_le_bytes());
        }
        for w in &self.weights {
            hasher.update(&w.to_bits().to_le_bytes());
        }
        *hasher.finalize().as_bytes()
    }

    /// Validates conserved protocol safety, parameter bounds, and BLAKE3 integrity.
    pub fn verify_integrity(&self) -> Result<(), DnaError> {
        if !self.conserved.is_valid() {
            return Err(DnaError::ConservedGeneViolation);
        }
        if !self.adaptive.is_finite_and_bounded() {
            return Err(DnaError::NonFiniteParameter);
        }
        let expected = self.adaptive.expected_weight_len();
        if self.weights.len() != expected {
            return Err(DnaError::WeightShapeMismatch { expected, actual: self.weights.len() });
        }
        if self.weights.iter().any(|w| !w.is_finite())
            || self.epigenetics.operon_gains.iter().any(|g| !g.is_finite())
        {
            return Err(DnaError::NonFiniteParameter);
        }
        if self.compute_hash() != self.dna_id {
            return Err(DnaError::ChecksumMismatch);
        }
        Ok(())
    }

    /// Epigenetic differentiation into a target histological role preserving weights.
    pub fn differentiate(&self, target_role: CellRole) -> Self {
        let mut next = self.clone();
        next.parent_dna_id = Some(self.dna_id);
        next.epigenetics = EpigeneticMask::differentiated(target_role);
        next.dna_id = next.compute_hash();
        next
    }

    /// Extracts `(w1, b1, w2, b2)` slices for the currently expressed role.
    pub fn active_network_slices(&self) -> (&[f32], &[f32], &[f32], &[f32]) {
        let (d_in, h, d_out) = (self.adaptive.input_dim as usize, self.adaptive.hidden_dim as usize, self.adaptive.output_dim as usize);
        let (w1_end, b1_end) = (d_in * h, d_in * h + h);
        let head_idx = self.epigenetics.expressed_role.operon_index().unwrap_or(0);
        let head_start = b1_end + head_idx * (h * d_out + d_out);
        let (w2_end, b2_end) = (head_start + h * d_out, head_start + h * d_out + d_out);
        (&self.weights[..w1_end], &self.weights[w1_end..b1_end], &self.weights[head_start..w2_end], &self.weights[w2_end..b2_end])
    }

    /// Replicates `1 -> 2` during Mitosis: Daughter A is faithful, Daughter B is mutated.
    pub fn mitosis_replicate<R: Rng>(&self, rng: &mut R) -> (Self, Self) {
        let mut daughter_a = self.clone();
        daughter_a.parent_dna_id = Some(self.dna_id);
        daughter_a.generation = self.generation.saturating_add(1);
        daughter_a.dna_id = daughter_a.compute_hash();

        let mut daughter_b = daughter_a.clone();
        let sigma = daughter_b.adaptive.mutation_sigma;
        for w in &mut daughter_b.weights {
            if rng.gen_bool(MITOSIS_WEIGHT_MUTATION_PROB) {
                *w = (*w + approx_standard_normal(rng) * sigma).clamp(-WEIGHT_CLAMP_LIMIT, WEIGHT_CLAMP_LIMIT);
            }
        }
        mutate_adaptive_genes(&mut daughter_b.adaptive, rng);
        daughter_b.dna_id = daughter_b.compute_hash();
        (daughter_a, daughter_b)
    }

    /// Horizontal Gene Transfer (Plasmid Crossover): blends recipient DNA with a high-fitness donor.
    pub fn horizontal_crossover<R: Rng>(&self, donor: &Self, rng: &mut R) -> Result<Self, DnaError> {
        self.verify_integrity()?;
        donor.verify_integrity()?;
        if !self.adaptive.has_same_shape(&donor.adaptive) {
            return Err(DnaError::IncompatibleCrossoverShape);
        }
        let mut child = self.clone();
        child.parent_dna_id = Some(self.dna_id);
        child.generation = self.generation.max(donor.generation).saturating_add(1);
        child.adaptive = blend_adaptive_genes(&self.adaptive, &donor.adaptive);
        for (cw, dw) in child.weights.iter_mut().zip(donor.weights.iter()) {
            if rng.gen_bool(CROSSOVER_WEIGHT_PROB) {
                *cw = (0.5 * *cw + 0.5 * *dw).clamp(-WEIGHT_CLAMP_LIMIT, WEIGHT_CLAMP_LIMIT);
            }
        }
        child.dna_id = child.compute_hash();
        Ok(child)
    }
}

fn update_genes_digest(hasher: &mut blake3::Hasher, c: &ConservedGenes, a: &AdaptiveGenes) {
    for val in [c.protocol_version, c.max_udp_mtu, c.min_quorum_bps] {
        hasher.update(&val.to_le_bytes());
    }
    hasher.update(&c.genesis_supply_tag);
    for dim in [a.input_dim, a.hidden_dim, a.output_dim] {
        hasher.update(&dim.to_le_bytes());
    }
    hasher.update(&a.leak_rate.to_bits().to_le_bytes());
    hasher.update(&a.mutation_sigma.to_bits().to_le_bytes());
    hasher.update(&[a.gossip_fanout]);
    hasher.update(&a.pulse_interval_ms.to_le_bytes());
    hasher.update(&a.fec_redundancy_bps.to_le_bytes());
}

fn mutate_adaptive_genes<R: Rng>(a: &mut AdaptiveGenes, rng: &mut R) {
    a.leak_rate = (a.leak_rate + approx_standard_normal(rng) * 0.03).clamp(MIN_LEAK_RATE, MAX_LEAK_RATE);
    a.mutation_sigma = (a.mutation_sigma + approx_standard_normal(rng) * 0.005).clamp(MITOSIS_SIGMA_MIN, MITOSIS_SIGMA_MAX);
    if rng.gen_bool(MITOSIS_TRAIT_MUTATION_PROB) {
        let step: i16 = if rng.gen_bool(0.5) { 1 } else { -1 };
        a.gossip_fanout = (a.gossip_fanout as i16 + step).clamp(MIN_GOSSIP_FANOUT as i16, MAX_GOSSIP_FANOUT as i16) as u8;
    }
    if rng.gen_bool(MITOSIS_TRAIT_MUTATION_PROB) {
        a.pulse_interval_ms = (a.pulse_interval_ms as i32 + rng.gen_range(-10..=10)).clamp(MIN_PULSE_INTERVAL_MS as i32, MAX_PULSE_INTERVAL_MS as i32) as u16;
    }
    if rng.gen_bool(MITOSIS_TRAIT_MUTATION_PROB) {
        a.fec_redundancy_bps = (a.fec_redundancy_bps as i32 + rng.gen_range(-50..=50)).clamp(MIN_FEC_REDUNDANCY_BPS as i32, MAX_FEC_REDUNDANCY_BPS as i32) as u16;
    }
}

fn blend_adaptive_genes(r: &AdaptiveGenes, d: &AdaptiveGenes) -> AdaptiveGenes {
    let mut b = r.clone();
    b.leak_rate = (RECIPIENT_BLEND_RATIO * r.leak_rate + DONOR_BLEND_RATIO * d.leak_rate).clamp(MIN_LEAK_RATE, MAX_LEAK_RATE);
    b.gossip_fanout = ((RECIPIENT_BLEND_RATIO * r.gossip_fanout as f32 + DONOR_BLEND_RATIO * d.gossip_fanout as f32).round() as u8).clamp(MIN_GOSSIP_FANOUT, MAX_GOSSIP_FANOUT);
    b.pulse_interval_ms = ((RECIPIENT_BLEND_RATIO * r.pulse_interval_ms as f32 + DONOR_BLEND_RATIO * d.pulse_interval_ms as f32).round() as u16).clamp(MIN_PULSE_INTERVAL_MS, MAX_PULSE_INTERVAL_MS);
    b.fec_redundancy_bps = ((RECIPIENT_BLEND_RATIO * r.fec_redundancy_bps as f32 + DONOR_BLEND_RATIO * d.fec_redundancy_bps as f32).round() as u16).clamp(MIN_FEC_REDUNDANCY_BPS, MAX_FEC_REDUNDANCY_BPS);
    b
}

fn approx_standard_normal<R: Rng>(rng: &mut R) -> f32 {
    (0..6).map(|_| rng.gen_range(-1.0f32..1.0f32)).sum::<f32>() * std::f32::consts::FRAC_1_SQRT_2
}
