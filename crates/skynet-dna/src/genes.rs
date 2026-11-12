//! Core genetic constants, conserved protocol invariants, adaptive traits,
//! and epigenetic differentiation masks for `MicroCell` genomes.

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_UDP_MTU_BYTES: u16 = 1200;
pub const MIN_SNOWBALL_QUORUM_BPS: u16 = 6667;
pub const MAX_BASIS_POINTS: u16 = 10_000;
pub const GENESIS_SUPPLY_TAG: [u8; 8] = *b"SKN21TRL";

pub const DEFAULT_INPUT_DIM: usize = 8;
pub const DEFAULT_HIDDEN_DIM: usize = 16;
pub const DEFAULT_OUTPUT_DIM: usize = 4;
pub const NUM_SPECIALIZED_OPERONS: usize = 4;

pub const MIN_LEAK_RATE: f32 = 0.05;
pub const MAX_LEAK_RATE: f32 = 0.95;
pub const DEFAULT_LEAK_RATE: f32 = 0.35;
pub const MIN_MUTATION_SIGMA: f32 = 0.001;
pub const MAX_MUTATION_SIGMA: f32 = 0.25;
pub const DEFAULT_MUTATION_SIGMA: f32 = 0.04;
pub const MIN_GOSSIP_FANOUT: u8 = 4;
pub const MAX_GOSSIP_FANOUT: u8 = 14;
pub const DEFAULT_GOSSIP_FANOUT: u8 = 6;
pub const MIN_PULSE_INTERVAL_MS: u16 = 30;
pub const MAX_PULSE_INTERVAL_MS: u16 = 150;
pub const DEFAULT_PULSE_INTERVAL_MS: u16 = 80;
pub const MIN_FEC_REDUNDANCY_BPS: u16 = 1050;
pub const MAX_FEC_REDUNDANCY_BPS: u16 = 1800;
pub const DEFAULT_FEC_REDUNDANCY_BPS: u16 = 1300;

const STEM_OPERON_GAIN: f32 = 0.25;
const INACTIVE_OPERON_GAIN: f32 = 0.05;
const ACTIVE_OPERON_GAIN: f32 = 1.0;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
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
    #[error("plasmid patch parent DNA hash mismatch")]
    PlasmidParentMismatch,
    #[error("plasmid patch canonical BLAKE3 digest mismatch")]
    PlasmidDigestMismatch,
    #[error("plasmid patch scale_milli must be strictly positive")]
    InvalidPlasmidScale,
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

    pub(crate) fn role_tag(self) -> u8 {
        self.operon_index().map_or(0, |idx| (idx + 1) as u8)
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
            && (MIN_SNOWBALL_QUORUM_BPS..=MAX_BASIS_POINTS).contains(&self.min_quorum_bps)
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
            leak_rate: DEFAULT_LEAK_RATE,
            mutation_sigma: DEFAULT_MUTATION_SIGMA,
            gossip_fanout: DEFAULT_GOSSIP_FANOUT,
            pulse_interval_ms: DEFAULT_PULSE_INTERVAL_MS,
            fec_redundancy_bps: DEFAULT_FEC_REDUNDANCY_BPS,
        }
    }
}

impl AdaptiveGenes {
    pub fn expected_weight_len(&self) -> usize {
        let (d_in, h, d_out) = (self.input_dim as usize, self.hidden_dim as usize, self.output_dim as usize);
        (d_in * h + h) + NUM_SPECIALIZED_OPERONS * (h * d_out + d_out)
    }

    pub fn active_operon_weight_len(&self) -> usize {
        let (d_in, h, d_out) = (self.input_dim as usize, self.hidden_dim as usize, self.output_dim as usize);
        (d_in * h + h) + (h * d_out + d_out)
    }

    pub fn has_same_shape(&self, other: &Self) -> bool {
        self.input_dim == other.input_dim
            && self.hidden_dim == other.hidden_dim
            && self.output_dim == other.output_dim
    }

    pub fn is_finite_and_bounded(&self) -> bool {
        self.input_dim > 0
            && self.hidden_dim > 0
            && self.output_dim > 0
            && self.leak_rate.is_finite()
            && (MIN_LEAK_RATE..=MAX_LEAK_RATE).contains(&self.leak_rate)
            && self.mutation_sigma.is_finite()
            && (MIN_MUTATION_SIGMA..=MAX_MUTATION_SIGMA).contains(&self.mutation_sigma)
            && (MIN_GOSSIP_FANOUT..=MAX_GOSSIP_FANOUT).contains(&self.gossip_fanout)
            && (MIN_PULSE_INTERVAL_MS..=MAX_PULSE_INTERVAL_MS).contains(&self.pulse_interval_ms)
            && (MIN_FEC_REDUNDANCY_BPS..=MAX_FEC_REDUNDANCY_BPS).contains(&self.fec_redundancy_bps)
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
        Self { expressed_role: CellRole::Stem, operon_gains: [STEM_OPERON_GAIN; NUM_SPECIALIZED_OPERONS] }
    }

    pub fn differentiated(role: CellRole) -> Self {
        let Some(idx) = role.operon_index() else { return Self::stem(); };
        let mut gains = [INACTIVE_OPERON_GAIN; NUM_SPECIALIZED_OPERONS];
        gains[idx] = ACTIVE_OPERON_GAIN;
        Self { expressed_role: role, operon_gains: gains }
    }
}
