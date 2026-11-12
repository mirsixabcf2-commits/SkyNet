//! `dag_ledger`: Cluster UTXO-DAG conflict tracking, principal covenants, and transaction finality.

use std::collections::{BTreeMap, BTreeSet, HashMap};

pub use crate::nano_skn::AgentMetabolicBank;
use crate::{
    CustodyPolicy, LedgerError, NanoSkn, SknTransaction, SynapticVoucher, TxOutput,
    UtreexoForestBuilder, UtxoId, GENESIS_TOTAL_NANO_SKN, MIN_TX_NANO_SKN,
};

fn hash_genesis_outputs(outputs: &[TxOutput]) -> Result<[u8; 32], LedgerError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"SKYNET_GENESIS_VERTEX");
    let mut total = NanoSkn::ZERO;
    for out in outputs {
        if out.amount.0 < MIN_TX_NANO_SKN {
            return Err(LedgerError::ZeroOutputAmount);
        }
        total = total.checked_add(out.amount)?;
        hasher.update(&out.owner_pubkey);
        hasher.update(&out.amount.0.to_le_bytes());
    }
    if total.0 > GENESIS_TOTAL_NANO_SKN {
        return Err(LedgerError::ExceedsGenesisCap);
    }
    Ok(*hasher.finalize().as_bytes())
}

fn validate_input_policy(
    utxo: &TxOutput,
    sender: &[u8; 32],
    fee: NanoSkn,
    protected: &mut BTreeMap<[u8; 32], NanoSkn>,
) -> Result<(), LedgerError> {
    match &utxo.policy {
        CustodyPolicy::DirectOwner if &utxo.owner_pubkey == sender => Ok(()),
        CustodyPolicy::DirectOwner => Err(LedgerError::UnauthorizedSpend),
        CustodyPolicy::AgentBond { human_owner, .. } if sender == human_owner => Ok(()),
        CustodyPolicy::AgentBond { human_owner, max_voucher_allowance } if sender == &utxo.owner_pubkey => {
            let max_spend = max_voucher_allowance.checked_add(fee)?;
            let min_ret = if utxo.amount > max_spend { utxo.amount.checked_sub(max_spend)? } else { NanoSkn::ZERO };
            let entry = protected.entry(*human_owner).or_insert(NanoSkn::ZERO);
            *entry = entry.checked_add(min_ret)?;
            Ok(())
        }
        CustodyPolicy::AgentBond { .. } => Err(LedgerError::UnauthorizedSpend),
    }
}

/// Cluster-level UTXO-DAG ledger tracking live UTXOs, conflict sets, and the Cluster Nutrient Pool.
#[derive(Debug, Clone)]
pub struct UtxoDagLedger {
    utxos: BTreeMap<UtxoId, TxOutput>,
    pending_txs: HashMap<[u8; 32], SknTransaction>,
    conflict_sets: HashMap<UtxoId, BTreeSet<[u8; 32]>>,
    finalized_tips: BTreeSet<[u8; 32]>,
    pub cluster_nutrient_pool: NanoSkn,
}

impl UtxoDagLedger {
    pub fn from_genesis(genesis_outputs: Vec<TxOutput>) -> Result<(Self, [u8; 32]), LedgerError> {
        let gh = hash_genesis_outputs(&genesis_outputs)?;
        let utxos = genesis_outputs.into_iter().enumerate().map(|(i, o)| (UtxoId { tx_hash: gh, vout: i as u16 }, o)).collect();
        let ledger = Self { utxos, pending_txs: HashMap::new(), conflict_sets: HashMap::new(), finalized_tips: BTreeSet::from([gh]), cluster_nutrient_pool: NanoSkn::ZERO };
        Ok((ledger, gh))
    }

    pub fn build_utreexo_forest(&self) -> UtreexoForestBuilder { UtreexoForestBuilder::from_utxos(self.utxos.iter()) }
    pub fn get_utxo(&self, id: &UtxoId) -> Option<&TxOutput> { self.utxos.get(id) }
    pub fn finalized_tips(&self) -> Vec<[u8; 32]> { self.finalized_tips.iter().copied().collect() }
    pub fn all_utxos(&self) -> Vec<(UtxoId, TxOutput)> { self.utxos.iter().map(|(k, v)| (*k, v.clone())).collect() }
    pub fn conflict_candidates(&self, utxo: &UtxoId) -> Option<&BTreeSet<[u8; 32]>> { self.conflict_sets.get(utxo) }
    pub fn pending_tx(&self, tx_hash: &[u8; 32]) -> Option<&SknTransaction> { self.pending_txs.get(tx_hash) }

    pub fn utxos_for_owner(&self, owner_pubkey: &[u8; 32]) -> Vec<(UtxoId, TxOutput)> {
        self.utxos.iter().filter(|(_, o)| &o.owner_pubkey == owner_pubkey).map(|(k, v)| (*k, v.clone())).collect()
    }

    pub fn balance_of(&self, owner_pubkey: &[u8; 32]) -> NanoSkn {
        NanoSkn(self.utxos.values().filter(|o| &o.owner_pubkey == owner_pubkey).map(|o| o.amount.0).sum())
    }

    pub fn human_total_net_worth(&self, human_pubkey: &[u8; 32]) -> NanoSkn {
        NanoSkn(self.utxos.values().filter(|o| &o.human_beneficiary() == human_pubkey).map(|o| o.amount.0).sum())
    }

    pub fn from_raw_state(utxos: Vec<(UtxoId, TxOutput)>, tips: Vec<[u8; 32]>, pool: NanoSkn) -> Self {
        Self { utxos: utxos.into_iter().collect(), pending_txs: HashMap::new(), conflict_sets: HashMap::new(), finalized_tips: tips.into_iter().collect(), cluster_nutrient_pool: pool }
    }

    fn verify_inputs(&self, tx: &SknTransaction) -> Result<(NanoSkn, BTreeMap<[u8; 32], NanoSkn>), LedgerError> {
        let mut seen = BTreeSet::new();
        let mut input_sum = NanoSkn::ZERO;
        let mut protected = BTreeMap::new();
        for inp in &tx.inputs {
            if !seen.insert(*inp) {
                return Err(LedgerError::DuplicateInput);
            }
            let utxo = self.utxos.get(inp).ok_or(LedgerError::UnknownOrSpentUtxo)?;
            validate_input_policy(utxo, &tx.sender_pubkey, tx.fee, &mut protected)?;
            input_sum = input_sum.checked_add(utxo.amount)?;
        }
        Ok((input_sum, protected))
    }

    fn verify_outputs(outputs: &[TxOutput], protected: &BTreeMap<[u8; 32], NanoSkn>) -> Result<NanoSkn, LedgerError> {
        let mut out_sum = NanoSkn::ZERO;
        let mut returned: BTreeMap<[u8; 32], NanoSkn> = BTreeMap::new();
        for out in outputs {
            if out.amount.0 < MIN_TX_NANO_SKN {
                return Err(LedgerError::ZeroOutputAmount);
            }
            out_sum = out_sum.checked_add(out.amount)?;
            let e = returned.entry(out.human_beneficiary()).or_insert(NanoSkn::ZERO);
            *e = e.checked_add(out.amount)?;
        }
        for (human, &min_req) in protected {
            if returned.get(human).copied().unwrap_or(NanoSkn::ZERO) < min_req {
                return Err(LedgerError::HumanPrincipalDiversionBlocked);
            }
        }
        Ok(out_sum)
    }

    fn register_pending(&mut self, tx: SknTransaction) -> ([u8; 32], Vec<[u8; 32]>) {
        let tx_hash = tx.tx_hash();
        let mut conflicts = BTreeSet::new();
        for inp in &tx.inputs {
            let entry = self.conflict_sets.entry(*inp).or_default();
            conflicts.extend(entry.iter().copied().filter(|&h| h != tx_hash));
            entry.insert(tx_hash);
        }
        self.pending_txs.insert(tx_hash, tx);
        (tx_hash, conflicts.into_iter().collect())
    }

    pub fn admit_pending_tx(&mut self, tx: SknTransaction) -> Result<([u8; 32], Vec<[u8; 32]>), LedgerError> {
        if tx.inputs.is_empty() || tx.outputs.is_empty() {
            return Err(LedgerError::EmptyInputsOrOutputs);
        }
        tx.verify_signature()?;
        let (in_sum, protected) = self.verify_inputs(&tx)?;
        let out_sum = Self::verify_outputs(&tx.outputs, &protected)?;
        if in_sum != out_sum.checked_add(tx.fee)? {
            return Err(LedgerError::ConservationMismatch { inputs: in_sum.0, outputs: out_sum.0, fee: tx.fee.0 });
        }
        Ok(self.register_pending(tx))
    }

    pub fn admit_unilateral_voucher_settlement(&mut self, tx: SknTransaction, v: &SynapticVoucher) -> Result<([u8; 32], Vec<[u8; 32]>), LedgerError> {
        if tx.inputs.len() != 1 || tx.outputs.is_empty() {
            return Err(LedgerError::EmptyInputsOrOutputs);
        }
        tx.verify_signature()?;
        if tx.inputs[0] != v.bond_utxo {
            return Err(LedgerError::UnknownOrSpentUtxo);
        }
        let bond = self.utxos.get(&v.bond_utxo).ok_or(LedgerError::UnknownOrSpentUtxo)?;
        let CustodyPolicy::AgentBond { human_owner, max_voucher_allowance } = bond.policy else {
            return Err(LedgerError::UnauthorizedSpend);
        };
        if bond.owner_pubkey != v.payer_agent_pubkey || tx.sender_pubkey != v.payee_pubkey || tx.outputs[0].owner_pubkey != v.payee_pubkey || tx.outputs[0].amount != v.cumulative_amount {
            return Err(LedgerError::UnauthorizedSpend);
        }
        v.verify(max_voucher_allowance)?;
        let refund = bond.amount.checked_sub(v.cumulative_amount.checked_add(tx.fee)?)?;
        if refund.0 > 0 && (tx.outputs.len() < 2 || tx.outputs[1].human_beneficiary() != human_owner || tx.outputs[1].amount != refund) {
            return Err(LedgerError::HumanPrincipalDiversionBlocked);
        }
        let out_sum = tx.outputs.iter().try_fold(NanoSkn::ZERO, |a, o| a.checked_add(o.amount))?;
        if bond.amount != out_sum.checked_add(tx.fee)? {
            return Err(LedgerError::ConservationMismatch { inputs: bond.amount.0, outputs: out_sum.0, fee: tx.fee.0 });
        }
        Ok(self.register_pending(tx))
    }

    fn purge_losing_conflicts(&mut self, losing_txs: &BTreeSet<[u8; 32]>) {
        for loser in losing_txs {
            let Some(ltx) = self.pending_txs.remove(loser) else { continue };
            for inp in &ltx.inputs {
                if let Some(set) = self.conflict_sets.get_mut(inp) {
                    set.remove(loser);
                }
            }
        }
    }

    pub fn finalize_tx(&mut self, tx_hash: &[u8; 32]) -> Result<Vec<UtxoId>, LedgerError> {
        let tx = self.pending_txs.remove(tx_hash).ok_or(LedgerError::UnknownTransaction)?;
        if tx.inputs.iter().any(|inp| !self.utxos.contains_key(inp)) {
            return Err(LedgerError::UnknownOrSpentUtxo);
        }
        let mut losers = BTreeSet::new();
        for inp in &tx.inputs {
            self.utxos.remove(inp);
            if let Some(cs) = self.conflict_sets.remove(inp) {
                losers.extend(cs.into_iter().filter(|c| c != tx_hash));
            }
        }
        self.purge_losing_conflicts(&losers);
        let minted = tx.outputs.into_iter().enumerate().map(|(i, out)| {
            let id = UtxoId { tx_hash: *tx_hash, vout: i as u16 };
            self.utxos.insert(id, out);
            id
        }).collect();
        for p in &tx.dag_parents {
            self.finalized_tips.remove(p);
        }
        self.finalized_tips.insert(*tx_hash);
        self.cluster_nutrient_pool = self.cluster_nutrient_pool.checked_add(tx.fee)?;
        Ok(minted)
    }

    pub fn total_tracked_supply(&self) -> Result<NanoSkn, LedgerError> {
        self.utxos.values().try_fold(self.cluster_nutrient_pool, |a, o| a.checked_add(o.amount))
    }
}
