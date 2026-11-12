//! `skynet-dna`: Genetic encoding, epigenetic differentiation, inheritance,
//! mutation, and horizontal plasmid crossover for SkyNet's micro-AI cells.

pub mod cell_dna;
pub mod genes;
pub mod plasmid_patch;

#[cfg(test)]
mod dna_tests;

pub use cell_dna::{CellDna, WEIGHT_CLAMP_LIMIT};
pub use genes::{
    AdaptiveGenes, CellRole, ConservedGenes, DnaError, EpigeneticMask, DEFAULT_FEC_REDUNDANCY_BPS,
    DEFAULT_GOSSIP_FANOUT, DEFAULT_HIDDEN_DIM, DEFAULT_INPUT_DIM, DEFAULT_LEAK_RATE,
    DEFAULT_MUTATION_SIGMA, DEFAULT_OUTPUT_DIM, DEFAULT_PULSE_INTERVAL_MS, GENESIS_SUPPLY_TAG,
    MAX_BASIS_POINTS, MAX_FEC_REDUNDANCY_BPS, MAX_GOSSIP_FANOUT, MAX_LEAK_RATE,
    MAX_MUTATION_SIGMA, MAX_PULSE_INTERVAL_MS, MAX_UDP_MTU_BYTES, MIN_FEC_REDUNDANCY_BPS,
    MIN_GOSSIP_FANOUT, MIN_LEAK_RATE, MIN_MUTATION_SIGMA, MIN_PULSE_INTERVAL_MS,
    MIN_SNOWBALL_QUORUM_BPS, NUM_SPECIALIZED_OPERONS, PROTOCOL_VERSION,
};
pub use plasmid_patch::{
    apply_plasmid_patch, PlasmidPatch, DEFAULT_PLASMID_SCALE_MILLI, PLASMID_PATCH_DOMAIN,
};
