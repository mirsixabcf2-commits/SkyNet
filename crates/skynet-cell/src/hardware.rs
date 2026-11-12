//! Hardware capacity tiers, metabolic thresholds, device selection, and `CellError`.

use candle_core::Device;
use serde::{Deserialize, Serialize};
use skynet_dna::DnaError;
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
