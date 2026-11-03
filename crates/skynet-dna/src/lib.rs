//! `skynet-dna`: Genetic encoding, epigenetic differentiation, inheritance,
//! mutation, and horizontal plasmid crossover for SkyNet's micro-AI cells.

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_UDP_MTU_BYTES: u16 = 1200;
pub const MIN_SNOWBALL_QUORUM_BPS: u16 = 6667;
pub const GENESIS_SUPPLY_TAG: [u8; 8] = *b"SKN21TRL";

pub const DEFAULT_INPUT_DIM: usize = 8;
pub const DEFAULT_HIDDEN_DIM: usize = 16;
pub const DEFAULT_OUTPUT_DIM: usize = 4;
pub const NUM_SPECIALIZED_OPERONS: usize = 4;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DnaError {
    #[error("conserved protocol genes violated — potential fork or corrupted genome")]
    ConservedGeneViolation,
    #[error("neural weight dimensions mismatch expected genotype shape (expected {expected}, got {actual})")]
    WeightShapeMismatch { expected: usize, actual: usize },
    #[error("non-finite weight or hyperparameter detected in DNA")]
    NonFiniteParameter,
    #[error("DNA BLAKE3 checksum mismatch")]
    ChecksumMismatch,
    #[error("incompatible neural shapes for horizontal gene crossover")]
    IncompatibleCrossoverShape,
}

/// Biological cell lineage inside a multicellular `AgentNode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CellRole {
    /// Pluripotent stem cell: dormant low-energy reserve capable of differentiating into any role.
    Stem,
    /// Skin & immune boundary cell: scores incoming lossy UDP packets for anomalies/toxins.
    Epidermal,
    /// Blood cell: regulates internal `nanoSKN` nutrient distribution and signal priority.
    Vascular,
    /// Organ cell (Heart/Liver/Kidney): drives Snowball consensus pulse, UTXO checks & autophagy.
    Visceral,
    /// Brain/cortical cell: performs cognitive inference, prediction, and Agent strategy.
    Neural,
}

impl CellRole {
    /// Index into the 4 specialized operon projection heads inside `CellDna`.
    pub fn operon_index(self) -> Option<usize> {
        match self {
            Self::Stem => None,
            Self::Epidermal => Some(0),
            Self::Vascular => Some(1),
            Self::Visceral => Some(2),
            Self::Neural => Some(3),
        }
    }

    pub const ALL_SPECIALIZED: [CellRole; 4] = [
        CellRole::Epidermal,
        CellRole::Vascular,
        CellRole::Visceral,
        CellRole::Neural,
    ];
}

/// Immutable genes protecting consensus safety and wire compatibility across all generations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConservedGenes {
    pub protocol_version: u16,
    pub max_udp_mtu: u16,
    pub min_quorum_bps: u16,
    pub genesis_supply_tag: [u8; 8],
}

impl Default for ConservedGenes {
    fn default() -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            max_udp_mtu: MAX_UDP_MTU_BYTES,
            min_quorum_bps: MIN_SNOWBALL_QUORUM_BPS,
            genesis_supply_tag: GENESIS_SUPPLY_TAG,
        }
    }
}

impl ConservedGenes {
    pub fn is_valid(&self) -> bool {
        self.protocol_version == PROTOCOL_VERSION
            && self.max_udp_mtu == MAX_UDP_MTU_BYTES
            && self.min_quorum_bps >= MIN_SNOWBALL_QUORUM_BPS
            && self.min_quorum_bps <= 10_000
            && self.genesis_supply_tag == GENESIS_SUPPLY_TAG
    }
}

/// Mutable neuro-evolutionary & networking genes subject to natural selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveGenes {
    pub input_dim: u16,
    pub hidden_dim: u16,
    pub output_dim: u16,
    /// Liquid time-constant leak rate `tau` in `[0.05, 0.95]`.
    pub leak_rate: f32,
    /// Standard deviation of Gaussian weight perturbations during mitosis.
    pub mutation_sigma: f32,
    /// UDP gossip fanout `[4, 14]`.
    pub gossip_fanout: u8,
    /// Consensus pulse cadence `[30, 150]` ms.
    pub pulse_interval_ms: u16,
    /// Fountain erasure-coding redundancy in basis points `[1050, 1800]` (1.05x .. 1.80x).
    pub fec_redundancy_bps: u16,
}

impl Default for AdaptiveGenes {
    fn default() -> Self {
        Self {
            input_dim: DEFAULT_INPUT_DIM as u16,
            hidden_dim: DEFAULT_HIDDEN_DIM as u16,
            output_dim: DEFAULT_OUTPUT_DIM as u16,
            leak_rate: 0.35,
            mutation_sigma: 0.04,
            gossip_fanout: 6,
            pulse_interval_ms: 80,
            fec_redundancy_bps: 1300,
        }
    }
}

impl AdaptiveGenes {
    pub fn expected_weight_len(&self) -> usize {
        let d_in = self.input_dim as usize;
        let h = self.hidden_dim as usize;
        let d_out = self.output_dim as usize;
        // Shared trunk (W1: d_in * h, b1: h) + 4 specialized operon heads (W2_k: h * d_out, b2_k: d_out)
        let trunk = d_in * h + h;
        let heads = NUM_SPECIALIZED_OPERONS * (h * d_out + d_out);
        trunk + heads
    }

    pub fn is_finite_and_bounded(&self) -> bool {
        self.input_dim > 0
            && self.hidden_dim > 0
            && self.output_dim > 0
            && self.leak_rate.is_finite()
            && (0.05..=0.95).contains(&self.leak_rate)
            && self.mutation_sigma.is_finite()
            && (0.001..=0.25).contains(&self.mutation_sigma)
            && (4..=14).contains(&self.gossip_fanout)
            && (30..=150).contains(&self.pulse_interval_ms)
            && (1050..=1800).contains(&self.fec_redundancy_bps)
    }
}

/// Epigenetic methylation switch controlling which cell type a genome expresses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpigeneticMask {
    pub expressed_role: CellRole,
    /// Expression gain per specialized operon `[Epidermal, Vascular, Visceral, Neural]`.
    pub operon_gains: [f32; NUM_SPECIALIZED_OPERONS],
}

impl EpigeneticMask {
    pub fn stem() -> Self {
        Self {
            expressed_role: CellRole::Stem,
            operon_gains: [0.25; NUM_SPECIALIZED_OPERONS],
        }
    }

    pub fn differentiated(role: CellRole) -> Self {
        match role.operon_index() {
            None => Self::stem(),
            Some(idx) => {
                let mut gains = [0.05; NUM_SPECIALIZED_OPERONS];
                gains[idx] = 1.0;
                Self {
                    expressed_role: role,
                    operon_gains: gains,
                }
            }
        }
    }
}

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
        let conserved = ConservedGenes::default();
        let adaptive = AdaptiveGenes::default();
        let epigenetics = EpigeneticMask::stem();

        let len = adaptive.expected_weight_len();
        let scale = (2.0 / (adaptive.input_dim as f32 + adaptive.hidden_dim as f32)).sqrt();
        let weights: Vec<f32> = (0..len)
            .map(|_| rng.gen_range(-scale..scale))
            .collect();

        let mut dna = Self {
            dna_id: [0u8; 32],
            parent_dna_id: None,
            generation: 0,
            conserved,
            adaptive,
            epigenetics,
            weights,
        };
        dna.dna_id = dna.compute_hash();
        dna
    }

    /// Computes a deterministic BLAKE3 digest over the entire genotype & epigenome.
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SKYNET_DNA_V1");
        if let Some(parent) = &self.parent_dna_id {
            hasher.update(&[1u8]);
            hasher.update(parent);
        } else {
            hasher.update(&[0u8]);
        }
        hasher.update(&self.generation.to_le_bytes());
        hasher.update(&self.conserved.protocol_version.to_le_bytes());
        hasher.update(&self.conserved.max_udp_mtu.to_le_bytes());
        hasher.update(&self.conserved.min_quorum_bps.to_le_bytes());
        hasher.update(&self.conserved.genesis_supply_tag);

        hasher.update(&self.adaptive.input_dim.to_le_bytes());
        hasher.update(&self.adaptive.hidden_dim.to_le_bytes());
        hasher.update(&self.adaptive.output_dim.to_le_bytes());
        hasher.update(&self.adaptive.leak_rate.to_bits().to_le_bytes());
        hasher.update(&self.adaptive.mutation_sigma.to_bits().to_le_bytes());
        hasher.update(&[self.adaptive.gossip_fanout]);
        hasher.update(&self.adaptive.pulse_interval_ms.to_le_bytes());
        hasher.update(&self.adaptive.fec_redundancy_bps.to_le_bytes());

        let role_tag = match self.epigenetics.expressed_role {
            CellRole::Stem => 0u8,
            CellRole::Epidermal => 1u8,
            CellRole::Vascular => 2u8,
            CellRole::Visceral => 3u8,
            CellRole::Neural => 4u8,
        };
        hasher.update(&[role_tag]);
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
            return Err(DnaError::WeightShapeMismatch {
                expected,
                actual: self.weights.len(),
            });
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

    /// Epigenetic differentiation: switches a Stem Cell (or transdifferentiates) into a target role
    /// while preserving all underlying genetic weights.
    pub fn differentiate(&self, target_role: CellRole) -> Self {
        let mut next = self.clone();
        next.parent_dna_id = Some(self.dna_id);
        next.epigenetics = EpigeneticMask::differentiated(target_role);
        next.dna_id = next.compute_hash();
        next
    }

    /// Extracts `(w1, b1, w2, b2)` slices for the currently expressed role.
    /// Stem cells use the first operon head scaled down for minimal basal maintenance.
    pub fn active_network_slices(&self) -> (&[f32], &[f32], &[f32], &[f32]) {
        let d_in = self.adaptive.input_dim as usize;
        let h = self.adaptive.hidden_dim as usize;
        let d_out = self.adaptive.output_dim as usize;

        let w1_end = d_in * h;
        let b1_end = w1_end + h;
        let w1 = &self.weights[0..w1_end];
        let b1 = &self.weights[w1_end..b1_end];

        let head_idx = self.epigenetics.expressed_role.operon_index().unwrap_or(0);
        let head_stride = h * d_out + d_out;
        let head_start = b1_end + head_idx * head_stride;
        let w2_end = head_start + h * d_out;
        let b2_end = w2_end + d_out;

        let w2 = &self.weights[head_start..w2_end];
        let b2 = &self.weights[w2_end..b2_end];
        (w1, b1, w2, b2)
    }

    /// Replicates `1 -> 2` during Mitosis:
    /// - Daughter A is a faithful genetic copy preserving proven parent traits.
    /// - Daughter B inherits the parent DNA plus bounded neuro-evolutionary mutations.
    pub fn mitosis_replicate<R: Rng>(&self, rng: &mut R) -> (Self, Self) {
        let mut daughter_a = self.clone();
        daughter_a.parent_dna_id = Some(self.dna_id);
        daughter_a.generation = self.generation.saturating_add(1);
        daughter_a.dna_id = daughter_a.compute_hash();

        let mut daughter_b = daughter_a.clone();
        let sigma = daughter_b.adaptive.mutation_sigma;

        // Mutate neural weights with bounded clamping so weights never explode
        for w in &mut daughter_b.weights {
            if rng.gen_bool(0.35) {
                let delta = approx_standard_normal(rng) * sigma;
                *w = (*w + delta).clamp(-4.0, 4.0);
            }
        }

        // Mutate adaptive networking & time-constant genes slightly
        let leak_delta = approx_standard_normal(rng) * 0.03;
        daughter_b.adaptive.leak_rate =
            (daughter_b.adaptive.leak_rate + leak_delta).clamp(0.05, 0.95);

        let sigma_delta = approx_standard_normal(rng) * 0.005;
        daughter_b.adaptive.mutation_sigma =
            (daughter_b.adaptive.mutation_sigma + sigma_delta).clamp(0.005, 0.15);

        if rng.gen_bool(0.25) {
            let step: i16 = if rng.gen_bool(0.5) { 1 } else { -1 };
            daughter_b.adaptive.gossip_fanout =
                (daughter_b.adaptive.gossip_fanout as i16 + step).clamp(4, 14) as u8;
        }
        if rng.gen_bool(0.25) {
            let step: i32 = rng.gen_range(-10..=10);
            daughter_b.adaptive.pulse_interval_ms =
                (daughter_b.adaptive.pulse_interval_ms as i32 + step).clamp(30, 150) as u16;
        }
        if rng.gen_bool(0.25) {
            let step: i32 = rng.gen_range(-50..=50);
            daughter_b.adaptive.fec_redundancy_bps =
                (daughter_b.adaptive.fec_redundancy_bps as i32 + step).clamp(1050, 1800) as u16;
        }

        daughter_b.dna_id = daughter_b.compute_hash();
        (daughter_a, daughter_b)
    }

    /// Horizontal Gene Transfer (Plasmid Crossover): blends recipient DNA with a high-fitness donor.
    pub fn horizontal_crossover<R: Rng>(
        &self,
        donor: &Self,
        rng: &mut R,
    ) -> Result<Self, DnaError> {
        self.verify_integrity()?;
        donor.verify_integrity()?;

        if self.adaptive.input_dim != donor.adaptive.input_dim
            || self.adaptive.hidden_dim != donor.adaptive.hidden_dim
            || self.adaptive.output_dim != donor.adaptive.output_dim
        {
            return Err(DnaError::IncompatibleCrossoverShape);
        }

        let mut child = self.clone();
        child.parent_dna_id = Some(self.dna_id);
        child.generation = self.generation.max(donor.generation).saturating_add(1);

        // Blend adaptive traits toward high-performing donor (70% recipient + 30% donor)
        child.adaptive.leak_rate =
            (0.7 * self.adaptive.leak_rate + 0.3 * donor.adaptive.leak_rate).clamp(0.05, 0.95);
        child.adaptive.gossip_fanout =
            ((0.7 * self.adaptive.gossip_fanout as f32 + 0.3 * donor.adaptive.gossip_fanout as f32)
                .round() as u8)
                .clamp(4, 14);
        child.adaptive.pulse_interval_ms = ((0.7 * self.adaptive.pulse_interval_ms as f32
            + 0.3 * donor.adaptive.pulse_interval_ms as f32)
            .round() as u16)
            .clamp(30, 150);
        child.adaptive.fec_redundancy_bps = ((0.7 * self.adaptive.fec_redundancy_bps as f32
            + 0.3 * donor.adaptive.fec_redundancy_bps as f32)
            .round() as u16)
            .clamp(1050, 1800);

        for (cw, dw) in child.weights.iter_mut().zip(donor.weights.iter()) {
            if rng.gen_bool(0.30) {
                *cw = (0.5 * *cw + 0.5 * *dw).clamp(-4.0, 4.0);
            }
        }

        child.dna_id = child.compute_hash();
        Ok(child)
    }
}

/// Fast zero-dependency Irwin-Hall approximation of standard normal `N(0, 1)`.
fn approx_standard_normal<R: Rng>(rng: &mut R) -> f32 {
    let mut sum = 0.0f32;
    for _ in 0..6 {
        sum += rng.gen_range(-1.0f32..1.0f32);
    }
    // Variance of U(-1, 1) is 1/3; sum of 6 has variance 2 -> divide by sqrt(2)
    sum * std::f32::consts::FRAC_1_SQRT_2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_stem_and_differentiation_preserve_integrity() {
        let stem = CellDna::genesis_stem(42);
        assert_eq!(stem.epigenetics.expressed_role, CellRole::Stem);
        assert!(stem.verify_integrity().is_ok());

        for role in CellRole::ALL_SPECIALIZED {
            let diff = stem.differentiate(role);
            assert_eq!(diff.epigenetics.expressed_role, role);
            assert_eq!(diff.parent_dna_id, Some(stem.dna_id));
            assert_eq!(diff.weights, stem.weights);
            assert!(diff.verify_integrity().is_ok());
        }
    }

    #[test]
    fn mitosis_preserves_daughter_a_and_mutates_daughter_b() {
        let mut rng = ChaCha8Rng::seed_from_u64(777);
        let parent = CellDna::genesis_stem(100).differentiate(CellRole::Neural);
        let (d_a, d_b) = parent.mitosis_replicate(&mut rng);

        assert!(d_a.verify_integrity().is_ok());
        assert!(d_b.verify_integrity().is_ok());
        assert_eq!(d_a.generation, 1);
        assert_eq!(d_b.generation, 1);
        assert_eq!(d_a.parent_dna_id, Some(parent.dna_id));
        assert_eq!(d_b.parent_dna_id, Some(parent.dna_id));
        assert_eq!(d_a.weights, parent.weights);
        assert_ne!(d_b.weights, parent.weights);
        assert_eq!(d_b.conserved, parent.conserved);
    }

    #[test]
    fn tampering_with_conserved_genes_or_weights_fails_verification() {
        let mut dna = CellDna::genesis_stem(9);
        dna.weights[0] += 0.1;
        assert_eq!(dna.verify_integrity(), Err(DnaError::ChecksumMismatch));

        dna.conserved.min_quorum_bps = 4000;
        dna.dna_id = dna.compute_hash();
        assert_eq!(dna.verify_integrity(), Err(DnaError::ConservedGeneViolation));
    }
}
