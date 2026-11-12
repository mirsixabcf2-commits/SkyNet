//! `genesis`: Deterministic Mainnet `GenesisConfig`, `GENESIS_HASH`, and `85% / 15%` 21T SKN invariants.

use serde::{Deserialize, Serialize};

use crate::{
    CustodyPolicy, GenesisHalvingVault, LedgerError, NanoSkn, TxOutput, UtxoDagLedger,
    DEFAULT_EPOCHS_PER_YEAR, GENESIS_TOTAL_NANO_SKN, MIN_TX_NANO_SKN, NANO_PER_SKN,
};

/// `85%` of `21,000,000,000,000 SKN` locked in `GenesisHalvingVault`: `17,850,000,000,000 SKN`.
pub const GENESIS_VAULT_SKN: u128 = 17_850_000_000_000;
pub const GENESIS_VAULT_NANO_SKN: u128 = GENESIS_VAULT_SKN * NANO_PER_SKN;

/// `15%` of `21,000,000,000,000 SKN` initial circulating supply: `3,150,000,000,000 SKN`.
pub const GENESIS_CIRCULATING_SKN: u128 = 3_150_000_000_000;
pub const GENESIS_CIRCULATING_NANO_SKN: u128 = GENESIS_CIRCULATING_SKN * NANO_PER_SKN;

pub const DEFAULT_MAINNET_NETWORK_NAME: &str = "skynet-mainnet-v1";
pub const GENESIS_HASH_DOMAIN: &[u8] = b"SKYNET_GENESIS_CONFIG_V1";

const BOOTSTRAP_AIRDROP_SKN: u128 = 1_000_000_000_000;
const ECOSYSTEM_SEED_SKN: u128 = 2_150_000_000_000;

fn default_direct_policy() -> CustodyPolicy {
    CustodyPolicy::DirectOwner
}

/// Macro supply split (`85%` Halving Vault + `15%` Initial Circulating == `21T SKN`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisAllocation {
    pub total_supply: NanoSkn,
    pub vault_allocation: NanoSkn,
    pub circulating_allocation: NanoSkn,
}

impl GenesisAllocation {
    pub const fn mainnet_default() -> Self {
        Self {
            total_supply: NanoSkn(GENESIS_TOTAL_NANO_SKN),
            vault_allocation: NanoSkn(GENESIS_VAULT_NANO_SKN),
            circulating_allocation: NanoSkn(GENESIS_CIRCULATING_NANO_SKN),
        }
    }
}

/// Single initial circulating UTXO declared in `genesis.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisUtxoEntry {
    #[serde(default)]
    pub label: String,
    pub owner_pubkey: [u8; 32],
    pub amount: NanoSkn,
    #[serde(default = "default_direct_policy")]
    pub policy: CustodyPolicy,
}

impl GenesisUtxoEntry {
    pub fn direct(label: impl Into<String>, owner_pubkey: [u8; 32], amount: NanoSkn) -> Self {
        Self { label: label.into(), owner_pubkey, amount, policy: CustodyPolicy::DirectOwner }
    }

    pub fn to_tx_output(&self) -> TxOutput {
        TxOutput { owner_pubkey: self.owner_pubkey, amount: self.amount, policy: self.policy.clone() }
    }
}

/// Immutable network birth specification producing the 32-byte `GENESIS_HASH`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub network_name: String,
    pub total_supply: NanoSkn,
    pub vault_allocation: NanoSkn,
    pub circulating_allocation: NanoSkn,
    pub initial_utxos: Vec<GenesisUtxoEntry>,
    pub annual_halving_epochs: u64,
}

impl GenesisConfig {
    pub const fn allocation(&self) -> GenesisAllocation {
        GenesisAllocation {
            total_supply: self.total_supply,
            vault_allocation: self.vault_allocation,
            circulating_allocation: self.circulating_allocation,
        }
    }

    pub fn genesis_hash(&self) -> [u8; 32] {
        compute_genesis_hash(self)
    }

    pub fn validate(&self) -> Result<(), LedgerError> {
        validate_genesis_invariants(self)
    }

    pub fn to_genesis_outputs(&self) -> Vec<TxOutput> {
        self.initial_utxos.iter().map(GenesisUtxoEntry::to_tx_output).collect()
    }

    pub fn build_ledger_and_vault(
        &self,
    ) -> Result<(UtxoDagLedger, GenesisHalvingVault, [u8; 32]), LedgerError> {
        validate_genesis_invariants(self)?;
        let vault = GenesisHalvingVault::new(self.circulating_allocation, self.annual_halving_epochs)?;
        let (ledger, _) = UtxoDagLedger::from_genesis(self.to_genesis_outputs())?;
        Ok((ledger, vault, compute_genesis_hash(self)))
    }
}

/// Builds a canonical `21T SKN` Genesis config for `network_name` and `bootstrap_owner`.
pub fn genesis_for_network(network_name: impl Into<String>, bootstrap_owner: [u8; 32]) -> GenesisConfig {
    let airdrop_pk = blake3::derive_key("skynet/genesis/marketing-bootstrap/v1", &bootstrap_owner);
    GenesisConfig {
        network_name: network_name.into(),
        total_supply: NanoSkn(GENESIS_TOTAL_NANO_SKN),
        vault_allocation: NanoSkn(GENESIS_VAULT_NANO_SKN),
        circulating_allocation: NanoSkn(GENESIS_CIRCULATING_NANO_SKN),
        initial_utxos: vec![
            GenesisUtxoEntry::direct("marketing_bootstrap_vault", airdrop_pk, NanoSkn(BOOTSTRAP_AIRDROP_SKN * NANO_PER_SKN)),
            GenesisUtxoEntry::direct("ecosystem_seed_treasury", bootstrap_owner, NanoSkn(ECOSYSTEM_SEED_SKN * NANO_PER_SKN)),
        ],
        annual_halving_epochs: DEFAULT_EPOCHS_PER_YEAR,
    }
}

/// Deterministic Mainnet `GenesisConfig` (`85%` `17.85T SKN` Vault + `15%` `3.15T SKN` Circulating).
pub fn default_mainnet_genesis() -> GenesisConfig {
    let treasury_pk = blake3::derive_key("skynet/genesis/mainnet-seed-treasury/v1", b"SKYNET_V1");
    genesis_for_network(DEFAULT_MAINNET_NETWORK_NAME, treasury_pk)
}

/// Computes canonical 32-byte `GENESIS_HASH` via BLAKE3 (`"SKYNET_GENESIS_CONFIG_V1"`).
pub fn compute_genesis_hash(config: &GenesisConfig) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(GENESIS_HASH_DOMAIN);
    h.update(&(config.network_name.len() as u32).to_le_bytes());
    h.update(config.network_name.as_bytes());
    h.update(&config.total_supply.0.to_le_bytes());
    h.update(&config.vault_allocation.0.to_le_bytes());
    h.update(&config.circulating_allocation.0.to_le_bytes());
    h.update(&config.annual_halving_epochs.to_le_bytes());
    h.update(&(config.initial_utxos.len() as u32).to_le_bytes());
    for entry in &config.initial_utxos {
        h.update(&(entry.label.len() as u32).to_le_bytes());
        h.update(entry.label.as_bytes());
        entry.to_tx_output().hash_into(&mut h);
    }
    *h.finalize().as_bytes()
}

/// Enforces exact `85%` (`17.85T SKN`) + `15%` (`3.15T SKN`) == `21,000,000,000,000 SKN` to `1 nanoSKN`.
pub fn validate_genesis_invariants(config: &GenesisConfig) -> Result<(), LedgerError> {
    if config.network_name.trim().is_empty() || config.annual_halving_epochs == 0 {
        return Err(LedgerError::InvalidGenesisMetadata);
    }
    if config.initial_utxos.is_empty() {
        return Err(LedgerError::EmptyInputsOrOutputs);
    }
    let mut utxo_sum = NanoSkn::ZERO;
    for entry in &config.initial_utxos {
        if entry.amount.0 < MIN_TX_NANO_SKN {
            return Err(LedgerError::ZeroOutputAmount);
        }
        utxo_sum = utxo_sum.checked_add(entry.amount)?;
    }
    let combined = config.vault_allocation.checked_add(utxo_sum)?;
    if config.total_supply.0 != GENESIS_TOTAL_NANO_SKN
        || config.vault_allocation.0 != GENESIS_VAULT_NANO_SKN
        || config.circulating_allocation.0 != GENESIS_CIRCULATING_NANO_SKN
        || utxo_sum != config.circulating_allocation
        || combined.0 != GENESIS_TOTAL_NANO_SKN
    {
        return Err(LedgerError::GenesisSupplyMismatch {
            vault: config.vault_allocation.0,
            circulating: utxo_sum.0,
            expected_total: GENESIS_TOTAL_NANO_SKN,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mainnet_genesis_preserves_exact_21t_supply_and_deterministic_hash() {
        let g1 = default_mainnet_genesis();
        let g2 = default_mainnet_genesis();
        assert!(validate_genesis_invariants(&g1).is_ok());
        assert_eq!(g1.allocation(), GenesisAllocation::mainnet_default());
        assert_eq!(compute_genesis_hash(&g1), compute_genesis_hash(&g2));

        let (ledger, vault, ghash) = g1.build_ledger_and_vault().unwrap();
        assert_eq!(ghash, g1.genesis_hash());
        assert_eq!(vault.total_accounted_supply().unwrap().0, GENESIS_TOTAL_NANO_SKN);
        assert_eq!(ledger.total_tracked_supply().unwrap().0, GENESIS_CIRCULATING_NANO_SKN);
    }

    #[test]
    fn genesis_rejects_even_single_nanoskn_deviation() {
        let mut plus_one = default_mainnet_genesis();
        plus_one.initial_utxos[0].amount.0 += 1;
        assert!(matches!(validate_genesis_invariants(&plus_one), Err(LedgerError::GenesisSupplyMismatch { .. })));

        let mut minus_one = default_mainnet_genesis();
        minus_one.initial_utxos[1].amount.0 -= 1;
        assert!(matches!(validate_genesis_invariants(&minus_one), Err(LedgerError::GenesisSupplyMismatch { .. })));

        let mut wrong_split = default_mainnet_genesis();
        wrong_split.vault_allocation.0 -= 1;
        wrong_split.circulating_allocation.0 += 1;
        wrong_split.initial_utxos[0].amount.0 += 1;
        assert!(matches!(validate_genesis_invariants(&wrong_split), Err(LedgerError::GenesisSupplyMismatch { .. })));
    }
}
