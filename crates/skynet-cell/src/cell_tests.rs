//! Unit tests for `MicroCell`, `TissueBatch`, and exact `NanoSkn` mitosis conservation.

use candle_core::Device;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use skynet_dna::{CellDna, CellRole};
use skynet_ledger::NanoSkn;

use crate::{HardwareProfile, HardwareTier, MicroCell, TissueBatch};

fn assert_slices_close(lhs: &[f32], rhs: &[f32], tol: f32) {
    assert_eq!(lhs.len(), rhs.len());
    for (a, b) in lhs.iter().zip(rhs.iter()) {
        assert!((a - b).abs() < tol, "slice mismatch: {a} vs {b}");
    }
}

#[test]
fn batched_tissue_tensor_matches_individual_cell_forward_exact() {
    let mut rng = ChaCha8Rng::seed_from_u64(2026);
    let device = Device::Cpu;
    let stem = CellDna::genesis_stem(99);
    let (_, mutated_a) = stem.differentiate(CellRole::Neural).mitosis_replicate(&mut rng);
    let (_, mutated_b) = stem.differentiate(CellRole::Epidermal).mitosis_replicate(&mut rng);

    let mut individual_cells = vec![
        MicroCell::spawn(1, stem, NanoSkn::from_nano(5_000)).expect("cell 1"),
        MicroCell::spawn(2, mutated_a, NanoSkn::from_nano(5_000)).expect("cell 2"),
        MicroCell::spawn(3, mutated_b, NanoSkn::from_nano(5_000)).expect("cell 3"),
    ];
    let mut batched_cells = individual_cells.clone();
    let inputs = vec![
        vec![0.1, -0.4, 0.8, 0.0, -0.2, 0.5, 0.3, -0.7],
        vec![-0.5, 0.9, 0.2, -0.1, 0.6, -0.3, 0.4, 0.2],
        vec![0.7, 0.1, -0.8, 0.3, 0.0, 0.9, -0.6, 0.5],
    ];

    let single_outs: Vec<Vec<f32>> = individual_cells
        .iter_mut()
        .zip(inputs.iter())
        .map(|(c, inp)| c.forward(inp, &device).expect("forward"))
        .collect();
    let batch_outs = TissueBatch::forward_slice(&mut batched_cells, &inputs, &device).expect("batch");

    for i in 0..3 {
        assert_slices_close(&single_outs[i], &batch_outs[i], 1e-5);
        assert_slices_close(&individual_cells[i].hidden_state, &batched_cells[i].hidden_state, 1e-5);
    }
}

#[test]
fn mitosis_conserves_exact_odd_nanoskn_without_rounding_loss() {
    let mut rng = ChaCha8Rng::seed_from_u64(55);
    let profile = HardwareProfile::for_tier(HardwareTier::TinyPeripheral);
    let initial_energy = NanoSkn::from_nano(3_333);
    let mut parent = MicroCell::spawn(
        1,
        CellDna::genesis_stem(7).differentiate(CellRole::Visceral),
        initial_energy,
    )
    .expect("spawn parent");

    let (daughter, blood_fee) = parent.divide_mitosis(2, &profile, &mut rng).expect("mitosis");
    let total_after = parent
        .energy
        .checked_add(daughter.energy)
        .and_then(|s| s.checked_add(blood_fee))
        .expect("no overflow");
    assert_eq!(total_after, initial_energy);
}
