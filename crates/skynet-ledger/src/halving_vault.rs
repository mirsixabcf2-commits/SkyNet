//! `halving_vault`: Fixed `21,000,000,000,000 SKN` Genesis supply and 50% annual halving schedule.

use serde::{Deserialize, Serialize};

use crate::{LedgerError, NanoSkn, GENESIS_TOTAL_NANO_SKN};

/// Enforces the fixed `21,000,000,000,000 SKN` Genesis supply and 50% annual halving schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisHalvingVault {
    pub epochs_per_year: u64,
    pub initial_seed_circulating: NanoSkn,
    pub initial_reserve_locked: NanoSkn,
    pub remaining_reserve: NanoSkn,
    pub distributed_from_reserve: NanoSkn,
    pub next_epoch_to_release: u64,
}

impl GenesisHalvingVault {
    pub fn new(seed_circulating: NanoSkn, epochs_per_year: u64) -> Result<Self, LedgerError> {
        if seed_circulating.0 > GENESIS_TOTAL_NANO_SKN {
            return Err(LedgerError::ExceedsGenesisCap);
        }
        let epochs = epochs_per_year.max(1);
        let reserve = NanoSkn(GENESIS_TOTAL_NANO_SKN - seed_circulating.0);
        Ok(Self {
            epochs_per_year: epochs,
            initial_seed_circulating: seed_circulating,
            initial_reserve_locked: reserve,
            remaining_reserve: reserve,
            distributed_from_reserve: NanoSkn::ZERO,
            next_epoch_to_release: 0,
        })
    }

    pub fn annual_quota_for_year(&self, year_index: u64) -> NanoSkn {
        let shift = year_index.saturating_add(1);
        if shift >= 128 {
            return NanoSkn::ZERO;
        }
        NanoSkn(self.initial_reserve_locked.0 >> shift)
    }

    pub fn scheduled_epoch_emission(&self, epoch: u64) -> NanoSkn {
        if self.remaining_reserve.0 == 0 {
            return NanoSkn::ZERO;
        }
        let year_index = epoch / self.epochs_per_year;
        let annual = self.annual_quota_for_year(year_index).0;
        let per_epoch = annual / (self.epochs_per_year as u128);
        NanoSkn(per_epoch.min(self.remaining_reserve.0))
    }

    pub fn release_next_epoch(&mut self) -> Result<NanoSkn, LedgerError> {
        let emission = self.scheduled_epoch_emission(self.next_epoch_to_release);
        self.remaining_reserve = self.remaining_reserve.checked_sub(emission)?;
        self.distributed_from_reserve = self.distributed_from_reserve.checked_add(emission)?;
        self.next_epoch_to_release = self.next_epoch_to_release.saturating_add(1);
        debug_assert_eq!(self.total_accounted_supply()?.0, GENESIS_TOTAL_NANO_SKN);
        Ok(emission)
    }

    pub fn total_accounted_supply(&self) -> Result<NanoSkn, LedgerError> {
        self.initial_seed_circulating
            .checked_add(self.distributed_from_reserve)?
            .checked_add(self.remaining_reserve)
    }
}
