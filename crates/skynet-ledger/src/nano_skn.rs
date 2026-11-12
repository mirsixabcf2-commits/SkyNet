//! `nano_skn`: Atomic `NanoSkn(u128)` monetary unit, 21T SKN constants, and Agent metabolic bank.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::LedgerError;

/// `1 SKN = 1,000,000,000 nanoSKN` (`10^9`).
pub const NANO_PER_SKN: u128 = 1_000_000_000;

/// Hard-capped total supply minted once at Genesis: `21,000,000,000,000 SKN` (21 Trillion SKN).
pub const GENESIS_MAX_SKN: u128 = 21_000_000_000_000;

/// Exact atomic Genesis supply in `nanoSKN`: `2.1 * 10^22 nanoSKN` (requires `u128`).
pub const GENESIS_TOTAL_NANO_SKN: u128 = GENESIS_MAX_SKN * NANO_PER_SKN;

/// Minimum transferable / metabolic unit: `1 nanoSKN`.
pub const MIN_TX_NANO_SKN: u128 = 1;

/// Default number of 1-minute epochs per calendar year (`365 * 24 * 60 = 525,600`).
pub const DEFAULT_EPOCHS_PER_YEAR: u64 = 525_600;

/// Type-safe atomic SKN quantity (`1 nanoSKN = 10^-9 SKN`) backed by `u128`.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct NanoSkn(pub u128);

impl NanoSkn {
    pub const ZERO: Self = Self(0);
    pub const ONE_NANO: Self = Self(1);
    pub const GENESIS_CAP: Self = Self(GENESIS_TOTAL_NANO_SKN);

    pub const fn from_nano(nano: u128) -> Self {
        Self(nano)
    }

    pub fn from_skn(skn: u128) -> Result<Self, LedgerError> {
        skn.checked_mul(NANO_PER_SKN).map(Self).ok_or(LedgerError::Overflow)
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self, LedgerError> {
        self.0.checked_add(rhs.0).map(Self).ok_or(LedgerError::Overflow)
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, LedgerError> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(LedgerError::InsufficientBalance { available: self.0, required: rhs.0 })
    }

    pub const fn whole_skn(self) -> u128 {
        self.0 / NANO_PER_SKN
    }

    pub const fn sub_nano(self) -> u128 {
        self.0 % NANO_PER_SKN
    }
}

impl fmt::Display for NanoSkn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:09} SKN", self.whole_skn(), self.sub_nano())
    }
}

/// Enforces the Two-Tier Isomorphic Conservation Law inside a single AI `AgentNode`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetabolicBank {
    pub blood_pool: NanoSkn,
}

impl AgentMetabolicBank {
    pub fn new(initial_blood: NanoSkn) -> Self {
        Self { blood_pool: initial_blood }
    }

    pub fn deposit_to_blood(&mut self, amount: NanoSkn) -> Result<(), LedgerError> {
        self.blood_pool = self.blood_pool.checked_add(amount)?;
        Ok(())
    }

    pub fn draw_from_blood(&mut self, amount: NanoSkn) -> Result<NanoSkn, LedgerError> {
        self.blood_pool = self.blood_pool.checked_sub(amount)?;
        Ok(amount)
    }

    pub fn verify_invariant(
        &self,
        expected_total: NanoSkn,
        cell_energies: impl IntoIterator<Item = NanoSkn>,
    ) -> Result<(), LedgerError> {
        let mut cells_sum = NanoSkn::ZERO;
        for e in cell_energies {
            cells_sum = cells_sum.checked_add(e)?;
        }
        let actual = self.blood_pool.checked_add(cells_sum)?;
        if actual != expected_total {
            return Err(LedgerError::CellularInvariantMismatch {
                on_chain: expected_total.0,
                blood: self.blood_pool.0,
                cells: cells_sum.0,
            });
        }
        Ok(())
    }
}
