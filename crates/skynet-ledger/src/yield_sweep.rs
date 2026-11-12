//! `yield_sweep`: Soul-Body Custody automatic net-yield sweep (`0%` burn) to Human Cold Wallet.

use std::collections::BTreeSet;

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

use crate::{
    CustodyPolicy, LedgerError, NanoSkn, SknTransaction, TxOutput, UtxoId, MIN_TX_NANO_SKN,
    NANO_PER_SKN,
};

/// Default automatic cold-wallet sweep trigger threshold: `50 SKN`.
pub const DEFAULT_AUTO_SWEEP_SKN: u128 = 50;
/// Default automatic cold-wallet sweep trigger threshold in atomic `nanoSKN`.
pub const DEFAULT_AUTO_SWEEP_NANO_SKN: u128 = DEFAULT_AUTO_SWEEP_SKN * NANO_PER_SKN;

/// Configuration governing automatic `AgentBond` surplus sweeping to the Human Owner's cold wallet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct YieldSweepConfig {
    pub human_owner: [u8; 32],
    pub operating_reserve: NanoSkn,
    pub auto_sweep_threshold: NanoSkn,
}

impl YieldSweepConfig {
    pub const fn new(human_owner: [u8; 32], operating_reserve: NanoSkn) -> Self {
        Self {
            human_owner,
            operating_reserve,
            auto_sweep_threshold: NanoSkn(DEFAULT_AUTO_SWEEP_NANO_SKN),
        }
    }
}

impl Default for YieldSweepConfig {
    fn default() -> Self {
        Self {
            human_owner: [0u8; 32],
            operating_reserve: NanoSkn::ZERO,
            auto_sweep_threshold: NanoSkn(DEFAULT_AUTO_SWEEP_NANO_SKN),
        }
    }
}

/// Prepared `Yield Sweep` transaction paired with exact swept/retained amounts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YieldSweepOrder {
    pub tx: SknTransaction,
    pub swept_yield: NanoSkn,
    pub retained_reserve: NanoSkn,
}

fn inspect_bond_utxos(
    agent_pubkey: &[u8; 32],
    bond_utxos: &[(UtxoId, TxOutput)],
    expected_human: &[u8; 32],
) -> Result<(Vec<UtxoId>, NanoSkn, NanoSkn), LedgerError> {
    let mut seen = BTreeSet::new();
    let mut inputs = Vec::with_capacity(bond_utxos.len());
    let mut total_bonded = NanoSkn::ZERO;
    let mut max_allowance = NanoSkn::ZERO;

    for (id, out) in bond_utxos {
        if !seen.insert(*id) {
            return Err(LedgerError::DuplicateInput);
        }
        let CustodyPolicy::AgentBond { human_owner, max_voucher_allowance } = out.policy else {
            return Err(LedgerError::UnauthorizedSpend);
        };
        if &human_owner != expected_human {
            return Err(LedgerError::HumanPrincipalDiversionBlocked);
        }
        if &out.owner_pubkey != agent_pubkey {
            return Err(LedgerError::UnauthorizedSpend);
        }
        total_bonded = total_bonded.checked_add(out.amount)?;
        max_allowance = max_allowance.max(max_voucher_allowance);
        inputs.push(*id);
    }
    Ok((inputs, total_bonded, max_allowance))
}

fn build_sweep_outputs(
    cfg: &YieldSweepConfig,
    agent_pubkey: [u8; 32],
    swept_yield: NanoSkn,
    max_allowance: NanoSkn,
) -> Vec<TxOutput> {
    let mut outputs = vec![TxOutput::direct(cfg.human_owner, swept_yield)];
    if cfg.operating_reserve.0 > 0 {
        let next_allowance = max_allowance.min(cfg.operating_reserve);
        outputs.push(TxOutput::agent_bond(cfg.human_owner, agent_pubkey, cfg.operating_reserve, next_allowance));
    }
    outputs
}

/// Plans a `YieldSweepOrder` when net `AgentBond` surplus reaches `operating_reserve + auto_sweep_threshold`.
pub fn plan_yield_sweep(
    agent_signing_key: &SigningKey,
    bond_utxos: &[(UtxoId, TxOutput)],
    cfg: &YieldSweepConfig,
    tx_nonce: u64,
) -> Result<Option<YieldSweepOrder>, LedgerError> {
    if bond_utxos.is_empty() {
        return Ok(None);
    }
    let agent_pubkey = agent_signing_key.verifying_key().to_bytes();
    let (inputs, total_bonded, max_allowance) =
        inspect_bond_utxos(&agent_pubkey, bond_utxos, &cfg.human_owner)?;

    let trigger_floor = cfg.operating_reserve.checked_add(cfg.auto_sweep_threshold)?;
    if total_bonded < trigger_floor || total_bonded <= cfg.operating_reserve {
        return Ok(None);
    }
    let swept_yield = total_bonded.checked_sub(cfg.operating_reserve)?;
    if swept_yield.0 < MIN_TX_NANO_SKN {
        return Ok(None);
    }

    let outputs = build_sweep_outputs(cfg, agent_pubkey, swept_yield, max_allowance);
    let dag_parents = inputs.iter().map(|u| u.tx_hash).collect::<BTreeSet<_>>().into_iter().collect();
    let tx = SknTransaction::new_signed(agent_signing_key, inputs, outputs, NanoSkn::ZERO, dag_parents, tx_nonce);
    Ok(Some(YieldSweepOrder { tx, swept_yield, retained_reserve: cfg.operating_reserve }))
}

/// Builds a `0%` metabolic-fee `SknTransaction` sweeping net yield above `operating_reserve` to `human_owner`.
pub fn build_yield_sweep_tx(
    agent_signing_key: &SigningKey,
    bond_utxos: &[(UtxoId, TxOutput)],
    cfg: &YieldSweepConfig,
    tx_nonce: u64,
) -> Result<Option<SknTransaction>, LedgerError> {
    Ok(plan_yield_sweep(agent_signing_key, bond_utxos, cfg, tx_nonce)?.map(|order| order.tx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HumanWallet, UtxoDagLedger};

    #[test]
    fn yield_sweep_transfers_surplus_zero_burn_and_preserves_sovereign_reclaim() {
        let human = HumanWallet::from_recovery_phrase("yield-sweep-owner-seed");
        let reserve = NanoSkn::from_skn(100).unwrap();
        let (agent_key, _) = human.authorize_agent(0, NanoSkn::from_skn(20).unwrap(), 500);
        let agent_pk = agent_key.verifying_key().to_bytes();

        let initial_bond = NanoSkn::from_skn(160).unwrap();
        let bond_out = TxOutput::agent_bond(human.human_pubkey(), agent_pk, initial_bond, NanoSkn::from_skn(20).unwrap());
        let (mut ledger, ghash) = UtxoDagLedger::from_genesis(vec![bond_out]).unwrap();
        let cfg = YieldSweepConfig::new(human.human_pubkey(), reserve);

        let below = [(UtxoId { tx_hash: ghash, vout: 0 }, TxOutput::agent_bond(human.human_pubkey(), agent_pk, NanoSkn::from_skn(140).unwrap(), reserve))];
        assert_eq!(build_yield_sweep_tx(&agent_key, &below, &cfg, 1).unwrap(), None);

        let sweep_tx = build_yield_sweep_tx(&agent_key, &ledger.utxos_for_owner(&agent_pk), &cfg, 2).unwrap().unwrap();
        assert_eq!(sweep_tx.fee, NanoSkn::ZERO);
        let (shash, _) = ledger.admit_pending_tx(sweep_tx).unwrap();
        let minted = ledger.finalize_tx(&shash).unwrap();

        assert_eq!(ledger.balance_of(&human.human_pubkey()), NanoSkn::from_skn(60).unwrap());
        assert_eq!(ledger.balance_of(&agent_pk), reserve);
        assert_eq!(ledger.cluster_nutrient_pool, NanoSkn::ZERO);
        assert_eq!(ledger.total_tracked_supply().unwrap(), initial_bond);

        let reclaim = human.build_sovereign_reclaim_tx(minted[1], reserve, NanoSkn::ONE_NANO, vec![shash], 3).unwrap();
        let (rhash, _) = ledger.admit_pending_tx(reclaim).unwrap();
        ledger.finalize_tx(&rhash).unwrap();
        assert_eq!(ledger.balance_of(&agent_pk), NanoSkn::ZERO);
        assert_eq!(ledger.total_tracked_supply().unwrap(), initial_bond);
    }

    #[test]
    fn yield_sweep_blocks_principal_diversion_to_third_party() {
        let human = HumanWallet::from_recovery_phrase("real-human-owner");
        let attacker = HumanWallet::from_recovery_phrase("third-party-attacker");
        let (agent_key, _) = human.authorize_agent(0, NanoSkn::from_skn(10).unwrap(), 100);
        let agent_pk = agent_key.verifying_key().to_bytes();

        let bond_utxos = [(
            UtxoId { tx_hash: [5u8; 32], vout: 0 },
            TxOutput::agent_bond(human.human_pubkey(), agent_pk, NanoSkn::from_skn(200).unwrap(), NanoSkn::from_skn(10).unwrap()),
        )];
        let malicious_cfg = YieldSweepConfig::new(attacker.human_pubkey(), NanoSkn::from_skn(100).unwrap());
        assert_eq!(
            build_yield_sweep_tx(&agent_key, &bond_utxos, &malicious_cfg, 1),
            Err(LedgerError::HumanPrincipalDiversionBlocked)
        );
    }
}
