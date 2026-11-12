//! Unit tests for `CellDna` genesis synthesis, epigenetic differentiation,
//! mitosis replication, and conserved gene tamper detection.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{CellDna, CellRole, DnaError};

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
