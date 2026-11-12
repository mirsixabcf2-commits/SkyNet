//! `MicroCell` liquid neural cell lifecycle and `TissueBatch` parallel tensor execution.

use candle_core::{Device, Tensor};
use rand::Rng;
use serde::{Deserialize, Serialize};
use skynet_dna::{CellDna, CellRole};
use skynet_ledger::NanoSkn;

use crate::hardware::{CellError, HardwareProfile};

const DORMANT_STEM_GAIN: f32 = 0.25;
const MIN_DIVISION_HEALTH: f32 = 0.65;
const APOPTOSIS_HEALTH_FLOOR: f32 = 0.15;

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
        Ok(Self { cell_id, dna, energy: initial_energy, health: 1.0, cumulative_fitness: 0.0, age_ticks: 0, hidden_state: vec![0.0; h] })
    }

    pub fn role(&self) -> CellRole {
        self.dna.epigenetics.expressed_role
    }

    pub fn role_gain(&self) -> f32 {
        self.role().operon_index().map_or(DORMANT_STEM_GAIN, |idx| self.dna.epigenetics.operon_gains[idx])
    }

    pub fn differentiate(&mut self, target_role: CellRole) {
        self.dna = self.dna.differentiate(target_role);
    }

    /// Basal metabolic tick: Stem cells burn only 20% of active cell metabolism.
    pub fn metabolize_tick(&mut self, base_burn: NanoSkn) -> NanoSkn {
        self.age_ticks = self.age_ticks.saturating_add(1);
        let burn_raw = if self.role() == CellRole::Stem { (base_burn.0 / 5).max(1) } else { base_burn.0 };
        let actual_burn = burn_raw.min(self.energy.0);
        self.energy = NanoSkn(self.energy.0 - actual_burn);
        if self.energy.0 == 0 {
            self.health = (self.health - 0.25).max(0.0);
        }
        NanoSkn(actual_burn)
    }

    pub fn absorb_energy(&mut self, reward: NanoSkn, quality_score: f32) -> Result<(), CellError> {
        self.energy = self.energy.checked_add(reward)?;
        self.cumulative_fitness += quality_score;
        self.health = (self.health + 0.05 * quality_score).clamp(0.0, 1.0);
        Ok(())
    }

    /// Executes a single-cell Liquid Micro-Neural forward pass on `candle_core::Device`.
    pub fn forward(&mut self, input: &[f32], device: &Device) -> Result<Vec<f32>, CellError> {
        let (d_in, h, d_out) = (self.dna.adaptive.input_dim as usize, self.dna.adaptive.hidden_dim as usize, self.dna.adaptive.output_dim as usize);
        if input.len() != d_in {
            return Err(CellError::InputDimMismatch { expected: d_in, actual: input.len() });
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
        let y = (new_h.matmul(&w2)?.broadcast_add(&b2)?.tanh()? * (self.role_gain() as f64))?;
        self.hidden_state = new_h.squeeze(0)?.to_vec1::<f32>()?;
        Ok(y.squeeze(0)?.to_vec1::<f32>()?)
    }

    pub fn can_divide(&self, profile: &HardwareProfile) -> bool {
        self.health >= MIN_DIVISION_HEALTH && self.energy >= profile.mitosis_threshold
    }

    /// Splits `1 -> 2` (`Mitosis`) conserving exact `u128` `NanoSkn` down to `1 nanoSKN`.
    pub fn divide_mitosis<R: Rng>(&mut self, daughter_cell_id: u64, profile: &HardwareProfile, rng: &mut R) -> Result<(MicroCell, NanoSkn), CellError> {
        let min_needed = profile.mitosis_threshold.0.max(profile.mitosis_blood_fee.0.saturating_add(2));
        if self.energy.0 < min_needed {
            return Err(CellError::InsufficientMitosisEnergy { available: self.energy.0, required: min_needed });
        }
        let fee = profile.mitosis_blood_fee;
        let dist = self.energy.0 - fee.0;
        let (energy_a, energy_b) = (NanoSkn(dist / 2 + (dist % 2)), NanoSkn(dist / 2));
        let (dna_a, dna_b) = self.dna.mitosis_replicate(rng);
        self.dna = dna_a;
        self.energy = energy_a;
        self.age_ticks = 0;

        let mut daughter_b = MicroCell::spawn(daughter_cell_id, dna_b, energy_b)?;
        daughter_b.hidden_state = self.hidden_state.clone();
        daughter_b.health = self.health;
        Ok((daughter_b, fee))
    }

    pub fn should_apoptose(&self, profile: &HardwareProfile) -> bool {
        self.energy < profile.apoptosis_floor || self.health <= APOPTOSIS_HEALTH_FLOOR
    }
}

/// High-throughput **Batched Tissue Tensor Engine** (`[N, D_in, H]` & `[N, H, D_out]`).
pub struct TissueBatch;

#[derive(Default)]
struct PackedTissue {
    dims: (usize, usize, usize, usize),
    x: Vec<f32>,
    w1: Vec<f32>,
    b1: Vec<f32>,
    prev_h: Vec<f32>,
    tau: Vec<f32>,
    one_minus_tau: Vec<f32>,
    w2: Vec<f32>,
    b2: Vec<f32>,
    gain: Vec<f32>,
}

impl TissueBatch {
    pub fn forward_slice(cells: &mut [MicroCell], inputs: &[Vec<f32>], device: &Device) -> Result<Vec<Vec<f32>>, CellError> {
        if cells.is_empty() {
            return Ok(Vec::new());
        }
        if cells.len() != inputs.len() {
            return Err(CellError::InputDimMismatch { expected: cells.len(), actual: inputs.len() });
        }
        let p = pack_tissue(cells, inputs)?;
        let (n, d_in, h, d_out) = p.dims;
        let x = Tensor::from_vec(p.x, (n, 1, d_in), device)?;
        let w1 = Tensor::from_vec(p.w1, (n, d_in, h), device)?;
        let b1 = Tensor::from_vec(p.b1, (n, 1, h), device)?;
        let prev_h = Tensor::from_vec(p.prev_h, (n, 1, h), device)?;
        let tau = Tensor::from_vec(p.tau, (n, 1, 1), device)?;
        let one_minus_tau = Tensor::from_vec(p.one_minus_tau, (n, 1, 1), device)?;
        let pre_h = (x.matmul(&w1)? + &b1)?.tanh()?;
        let new_h = (prev_h.broadcast_mul(&one_minus_tau)? + pre_h.broadcast_mul(&tau)?)?;
        let w2 = Tensor::from_vec(p.w2, (n, h, d_out), device)?;
        let b2 = Tensor::from_vec(p.b2, (n, 1, d_out), device)?;
        let gain = Tensor::from_vec(p.gain, (n, 1, 1), device)?;
        let y = (new_h.matmul(&w2)? + &b2)?.tanh()?.broadcast_mul(&gain)?;
        for (cell, next_state) in cells.iter_mut().zip(new_h.squeeze(1)?.to_vec2::<f32>()?) {
            cell.hidden_state = next_state;
        }
        Ok(y.squeeze(1)?.to_vec2::<f32>()?)
    }
}

fn pack_tissue(cells: &[MicroCell], inputs: &[Vec<f32>]) -> Result<PackedTissue, CellError> {
    let n = cells.len();
    let a = &cells[0].dna.adaptive;
    let (d_in, h, d_out) = (a.input_dim as usize, a.hidden_dim as usize, a.output_dim as usize);
    let mut p = PackedTissue { dims: (n, d_in, h, d_out), ..Default::default() };
    for (cell, inp) in cells.iter().zip(inputs.iter()) {
        if !cell.dna.adaptive.has_same_shape(a) {
            return Err(CellError::HeterogeneousTissueShape);
        }
        if inp.len() != d_in {
            return Err(CellError::InputDimMismatch { expected: d_in, actual: inp.len() });
        }
        let (w1, b1, w2, b2) = cell.dna.active_network_slices();
        p.x.extend_from_slice(inp);
        p.w1.extend_from_slice(w1);
        p.b1.extend_from_slice(b1);
        p.prev_h.extend_from_slice(&cell.hidden_state);
        p.tau.push(cell.dna.adaptive.leak_rate);
        p.one_minus_tau.push(1.0 - cell.dna.adaptive.leak_rate);
        p.w2.extend_from_slice(w2);
        p.b2.extend_from_slice(b2);
        p.gain.push(cell.role_gain());
    }
    Ok(p)
}
