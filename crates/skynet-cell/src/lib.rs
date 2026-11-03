//! `skynet-cell`: Ultra-mini Liquid Neural AI cells powered by `candle-core`,
//! supporting Stem Cell differentiation, `1 -> 2` Mitosis, Neuro-evolution, Apoptosis,
//! and elastic scaling from Tiny CPU peripherals to Colossal GPU Batched Tissue Tensors.

use candle_core::{Device, Tensor};
use rand::Rng;
use serde::{Deserialize, Serialize};
use skynet_dna::{CellDna, CellRole, DnaError};
use skynet_ledger::{LedgerError, NanoSkn};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CellError {
    #[error("DNA error: {0}")]
    Dna(#[from] DnaError),
    #[error("Ledger/energy error: {0}")]
    Energy(#[from] LedgerError),
    #[error("Candle tensor compute error: {0}")]
    TensorCompute(#[from] candle_core::Error),
    #[error("input length mismatch: expected {expected}, got {actual}")]
    InputDimMismatch { expected: usize, actual: usize },
    #[error("insufficient cell energy for mitosis (has {available}, needs {required})")]
    InsufficientMitosisEnergy { available: u128, required: u128 },
    #[error("cells in a batched tissue execution must share neural dimensions")]
    HeterogeneousTissueShape,
}

/// Hardware capacity tier determining whether a Node grows into a Tiny or Colossal AI Agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareTier {
    /// Edge/IoT peripheral or weak CPU: lean survival organism (`<= 32` cells).
    TinyPeripheral,
    /// Personal laptop or standard workstation (`<= 512` cells).
    StandardNode,
    /// High-core server or GPU accelerator (`<= 16,384` cells batched in parallel).
    ColossalOrganism,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub tier: HardwareTier,
    pub max_cells: usize,
    pub basal_burn_per_tick: NanoSkn,
    pub mitosis_threshold: NanoSkn,
    pub mitosis_blood_fee: NanoSkn,
    pub apoptosis_floor: NanoSkn,
}

impl HardwareProfile {
    pub fn for_tier(tier: HardwareTier) -> Self {
        match tier {
            HardwareTier::TinyPeripheral => Self {
                tier,
                max_cells: 32,
                basal_burn_per_tick: NanoSkn::from_nano(50),
                mitosis_threshold: NanoSkn::from_nano(2_000),
                mitosis_blood_fee: NanoSkn::from_nano(100),
                apoptosis_floor: NanoSkn::from_nano(100),
            },
            HardwareTier::StandardNode => Self {
                tier,
                max_cells: 512,
                basal_burn_per_tick: NanoSkn::from_nano(25),
                mitosis_threshold: NanoSkn::from_nano(1_500),
                mitosis_blood_fee: NanoSkn::from_nano(60),
                apoptosis_floor: NanoSkn::from_nano(80),
            },
            HardwareTier::ColossalOrganism => Self {
                tier,
                max_cells: 16_384,
                basal_burn_per_tick: NanoSkn::from_nano(10),
                mitosis_threshold: NanoSkn::from_nano(1_000),
                mitosis_blood_fee: NanoSkn::from_nano(40),
                apoptosis_floor: NanoSkn::from_nano(50),
            },
        }
    }

    /// Selects the optimal `candle_core::Device` available on the host hardware.
    pub fn select_compute_device() -> Device {
        #[cfg(feature = "cuda")]
        if let Ok(dev) = Device::new_cuda(0) {
            return dev;
        }
        #[cfg(feature = "metal")]
        if let Ok(dev) = Device::new_metal(0) {
            return dev;
        }
        Device::Cpu
    }
}

/// A single living ultra-mini AI model inside an `AgentNode`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroCell {
    pub cell_id: u64,
    pub dna: CellDna,
    pub energy: NanoSkn,
    pub health: f32,
    pub cumulative_fitness: f32,
    pub age_ticks: u64,
    /// Recurrent Liquid Time-Constant short-term memory vector `h_t`.
    pub hidden_state: Vec<f32>,
}

impl MicroCell {
    /// Spawns a new `MicroCell` after verifying genetic integrity.
    pub fn spawn(cell_id: u64, dna: CellDna, initial_energy: NanoSkn) -> Result<Self, CellError> {
        dna.verify_integrity()?;
        let h = dna.adaptive.hidden_dim as usize;
        Ok(Self {
            cell_id,
            dna,
            energy: initial_energy,
            health: 1.0,
            cumulative_fitness: 0.0,
            age_ticks: 0,
            hidden_state: vec![0.0; h],
        })
    }

    pub fn role(&self) -> CellRole {
        self.dna.epigenetics.expressed_role
    }

    pub fn role_gain(&self) -> f32 {
        match self.role().operon_index() {
            None => 0.25, // Dormant Stem Cell gain
            Some(idx) => self.dna.epigenetics.operon_gains[idx],
        }
    }

    /// Differentiates a `Stem` cell (or transdifferentiates) into a specialized histological role.
    pub fn differentiate(&mut self, target_role: CellRole) {
        self.dna = self.dna.differentiate(target_role);
    }

    /// Basal metabolic tick: Stem cells burn only 20% of active cell metabolism!
    /// Returns the exact `NanoSkn` burned (to be recycled into the Agent's BloodPool or Nutrient Pool).
    pub fn metabolize_tick(&mut self, base_burn: NanoSkn) -> NanoSkn {
        self.age_ticks = self.age_ticks.saturating_add(1);
        let burn_raw = if self.role() == CellRole::Stem {
            (base_burn.0 / 5).max(1)
        } else {
            base_burn.0
        };
        let actual_burn = burn_raw.min(self.energy.0);
        self.energy = NanoSkn(self.energy.0 - actual_burn);
        if self.energy.0 == 0 {
            self.health = (self.health - 0.25).max(0.0);
        }
        NanoSkn(actual_burn)
    }

    /// Rewards the cell with `NanoSkn` drawn from the Agent's BloodPool and boosts its fitness.
    pub fn absorb_energy(&mut self, reward: NanoSkn, quality_score: f32) -> Result<(), CellError> {
        self.energy = self.energy.checked_add(reward)?;
        self.cumulative_fitness += quality_score;
        self.health = (self.health + 0.05 * quality_score).clamp(0.0, 1.0);
        Ok(())
    }

    /// Executes a single-cell Liquid Micro-Neural forward pass on `candle_core::Device`.
    /// Dynamics:
    /// `h_t = (1 - tau) * h_{t-1} + tau * tanh(x_t @ W1 + b1)`
    /// `y_t = gain * tanh(h_t @ W2 + b2)`
    pub fn forward(&mut self, input: &[f32], device: &Device) -> Result<Vec<f32>, CellError> {
        let d_in = self.dna.adaptive.input_dim as usize;
        let h = self.dna.adaptive.hidden_dim as usize;
        let d_out = self.dna.adaptive.output_dim as usize;

        if input.len() != d_in {
            return Err(CellError::InputDimMismatch {
                expected: d_in,
                actual: input.len(),
            });
        }

        let (w1_s, b1_s, w2_s, b2_s) = self.dna.active_network_slices();

        let x = Tensor::from_slice(input, (1, d_in), device)?;
        let w1 = Tensor::from_slice(w1_s, (d_in, h), device)?;
        let b1 = Tensor::from_slice(b1_s, (1, h), device)?;
        let prev_h = Tensor::from_slice(&self.hidden_state, (1, h), device)?;

        let pre_h = x.matmul(&w1)?.broadcast_add(&b1)?.tanh()?;
        let tau = self.dna.adaptive.leak_rate as f64;
        let new_h = ((prev_h * (1.0 - tau))? + (pre_h * tau)?)?;

        let w2 = Tensor::from_slice(w2_s, (h, d_out), device)?;
        let b2 = Tensor::from_slice(b2_s, (1, d_out), device)?;
        let gain = self.role_gain() as f64;
        let y = (new_h.matmul(&w2)?.broadcast_add(&b2)?.tanh()? * gain)?;

        self.hidden_state = new_h.squeeze(0)?.to_vec1::<f32>()?;
        Ok(y.squeeze(0)?.to_vec1::<f32>()?)
    }

    /// Checks whether this cell has enough `NanoSkn` and health to divide `1 -> 2`.
    pub fn can_divide(&self, profile: &HardwareProfile) -> bool {
        self.health >= 0.65 && self.energy >= profile.mitosis_threshold
    }

    /// Splits `1 -> 2` (`Mitosis`):
    /// - Mutates `self` into Daughter A (faithful DNA copy, ~50% post-fee energy).
    /// - Returns `(Daughter B [mutated DNA, ~50% post-fee energy], fee_returned_to_blood)`.
    /// - Exact `u128` conservation: `self_after + daughter_b + fee == self_before` down to `1 nanoSKN`!
    pub fn divide_mitosis<R: Rng>(
        &mut self,
        daughter_cell_id: u64,
        profile: &HardwareProfile,
        rng: &mut R,
    ) -> Result<(MicroCell, NanoSkn), CellError> {
        let min_needed = profile
            .mitosis_threshold
            .0
            .max(profile.mitosis_blood_fee.0.saturating_add(2));
        if self.energy.0 < min_needed {
            return Err(CellError::InsufficientMitosisEnergy {
                available: self.energy.0,
                required: min_needed,
            });
        }

        let fee = profile.mitosis_blood_fee;
        let distributable = self.energy.0 - fee.0;
        let energy_a = NanoSkn(distributable / 2 + (distributable % 2));
        let energy_b = NanoSkn(distributable / 2);

        let (dna_a, dna_b) = self.dna.mitosis_replicate(rng);
        self.dna = dna_a;
        self.energy = energy_a;
        self.age_ticks = 0;

        let mut daughter_b = MicroCell::spawn(daughter_cell_id, dna_b, energy_b)?;
        // Daughter B also inherits short-term cellular state continuity
        daughter_b.hidden_state = self.hidden_state.clone();
        daughter_b.health = self.health;

        Ok((daughter_b, fee))
    }

    /// Determines whether the cell should undergo Apoptosis (programmed elimination).
    pub fn should_apoptose(&self, profile: &HardwareProfile) -> bool {
        self.energy < profile.apoptosis_floor || self.health <= 0.15
    }
}

/// High-throughput **Batched Tissue Tensor Engine** (`[N, D_in, H]` & `[N, H, D_out]`):
/// Executes hundreds or thousands of distinct evolved `MicroCell` models in a single `candle`
/// batched matrix multiplication kernel on CPU or GPU (`Metal` / `CUDA`).
pub struct TissueBatch;

impl TissueBatch {
    pub fn forward_slice(
        cells: &mut [MicroCell],
        inputs: &[Vec<f32>],
        device: &Device,
    ) -> Result<Vec<Vec<f32>>, CellError> {
        if cells.is_empty() {
            return Ok(Vec::new());
        }
        assert_eq!(cells.len(), inputs.len());

        let n = cells.len();
        let d_in = cells[0].dna.adaptive.input_dim as usize;
        let h = cells[0].dna.adaptive.hidden_dim as usize;
        let d_out = cells[0].dna.adaptive.output_dim as usize;

        let mut x_flat = Vec::with_capacity(n * d_in);
        let mut w1_flat = Vec::with_capacity(n * d_in * h);
        let mut b1_flat = Vec::with_capacity(n * h);
        let mut prev_h_flat = Vec::with_capacity(n * h);
        let mut tau_flat = Vec::with_capacity(n);
        let mut one_minus_tau_flat = Vec::with_capacity(n);
        let mut w2_flat = Vec::with_capacity(n * h * d_out);
        let mut b2_flat = Vec::with_capacity(n * d_out);
        let mut gain_flat = Vec::with_capacity(n);

        for (cell, inp) in cells.iter().zip(inputs.iter()) {
            if cell.dna.adaptive.input_dim as usize != d_in
                || cell.dna.adaptive.hidden_dim as usize != h
                || cell.dna.adaptive.output_dim as usize != d_out
            {
                return Err(CellError::HeterogeneousTissueShape);
            }
            if inp.len() != d_in {
                return Err(CellError::InputDimMismatch {
                    expected: d_in,
                    actual: inp.len(),
                });
            }
            let (w1, b1, w2, b2) = cell.dna.active_network_slices();
            x_flat.extend_from_slice(inp);
            w1_flat.extend_from_slice(w1);
            b1_flat.extend_from_slice(b1);
            prev_h_flat.extend_from_slice(&cell.hidden_state);
            let tau = cell.dna.adaptive.leak_rate;
            tau_flat.push(tau);
            one_minus_tau_flat.push(1.0 - tau);
            w2_flat.extend_from_slice(w2);
            b2_flat.extend_from_slice(b2);
            gain_flat.push(cell.role_gain());
        }

        let x = Tensor::from_vec(x_flat, (n, 1, d_in), device)?;
        let w1 = Tensor::from_vec(w1_flat, (n, d_in, h), device)?;
        let b1 = Tensor::from_vec(b1_flat, (n, 1, h), device)?;
        let prev_h = Tensor::from_vec(prev_h_flat, (n, 1, h), device)?;
        let tau = Tensor::from_vec(tau_flat, (n, 1, 1), device)?;
        let one_minus_tau = Tensor::from_vec(one_minus_tau_flat, (n, 1, 1), device)?;

        let pre_h = (x.matmul(&w1)? + &b1)?.tanh()?;
        let new_h = (prev_h.broadcast_mul(&one_minus_tau)? + pre_h.broadcast_mul(&tau)?)?;

        let w2 = Tensor::from_vec(w2_flat, (n, h, d_out), device)?;
        let b2 = Tensor::from_vec(b2_flat, (n, 1, d_out), device)?;
        let gain = Tensor::from_vec(gain_flat, (n, 1, 1), device)?;

        let y = (new_h.matmul(&w2)? + &b2)?.tanh()?.broadcast_mul(&gain)?;

        let new_h_2d = new_h.squeeze(1)?.to_vec2::<f32>()?;
        for (cell, next_state) in cells.iter_mut().zip(new_h_2d.into_iter()) {
            cell.hidden_state = next_state;
        }

        Ok(y.squeeze(1)?.to_vec2::<f32>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn batched_tissue_tensor_matches_individual_cell_forward_exact() {
        let mut rng = ChaCha8Rng::seed_from_u64(2026);
        let device = Device::Cpu;

        let stem = CellDna::genesis_stem(99);
        let (_, mutated_a) = stem.differentiate(CellRole::Neural).mitosis_replicate(&mut rng);
        let (_, mutated_b) = stem
            .differentiate(CellRole::Epidermal)
            .mitosis_replicate(&mut rng);

        let mut individual_cells = vec![
            MicroCell::spawn(1, stem, NanoSkn::from_nano(5_000)).unwrap(),
            MicroCell::spawn(2, mutated_a, NanoSkn::from_nano(5_000)).unwrap(),
            MicroCell::spawn(3, mutated_b, NanoSkn::from_nano(5_000)).unwrap(),
        ];
        let mut batched_cells = individual_cells.clone();

        let inputs = vec![
            vec![0.1, -0.4, 0.8, 0.0, -0.2, 0.5, 0.3, -0.7],
            vec![-0.5, 0.9, 0.2, -0.1, 0.6, -0.3, 0.4, 0.2],
            vec![0.7, 0.1, -0.8, 0.3, 0.0, 0.9, -0.6, 0.5],
        ];

        let mut single_outs = Vec::new();
        for (c, inp) in individual_cells.iter_mut().zip(inputs.iter()) {
            single_outs.push(c.forward(inp, &device).unwrap());
        }

        let batch_outs = TissueBatch::forward_slice(&mut batched_cells, &inputs, &device).unwrap();

        for i in 0..3 {
            for (a, b) in single_outs[i].iter().zip(batch_outs[i].iter()) {
                assert!((a - b).abs() < 1e-5, "output mismatch: {a} vs {b}");
            }
            for (ha, hb) in individual_cells[i]
                .hidden_state
                .iter()
                .zip(batched_cells[i].hidden_state.iter())
            {
                assert!((ha - hb).abs() < 1e-5, "hidden mismatch: {ha} vs {hb}");
            }
        }
    }

    #[test]
    fn mitosis_conserves_exact_odd_nanoskn_without_rounding_loss() {
        let mut rng = ChaCha8Rng::seed_from_u64(55);
        let profile = HardwareProfile::for_tier(HardwareTier::TinyPeripheral);
        let initial_energy = NanoSkn::from_nano(3_333); // Odd nanoSKN!
        let mut parent = MicroCell::spawn(
            1,
            CellDna::genesis_stem(7).differentiate(CellRole::Visceral),
            initial_energy,
        )
        .unwrap();

        let (daughter, blood_fee) = parent.divide_mitosis(2, &profile, &mut rng).unwrap();
        let total_after = parent
            .energy
            .checked_add(daughter.energy)
            .unwrap()
            .checked_add(blood_fee)
            .unwrap();
        assert_eq!(total_after, initial_energy);
    }
}
