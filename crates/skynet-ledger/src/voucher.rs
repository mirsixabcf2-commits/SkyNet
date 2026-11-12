//! `voucher`: Loss-tolerant off-chain `SynapticVoucher` micro-payments and channel settlement.

use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};

use crate::custody::verify_ed25519;
use crate::{CustodyPolicy, LedgerError, NanoSkn, SknTransaction, TxOutput, UtxoId, MIN_TX_NANO_SKN};

/// Loss-tolerant cumulative off-chain micro-payment voucher over UDP (`<= 200 bytes`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynapticVoucher {
    pub bond_utxo: UtxoId,
    pub payer_agent_pubkey: [u8; 32],
    pub payee_pubkey: [u8; 32],
    pub sequence: u64,
    pub cumulative_amount: NanoSkn,
    pub signature: Vec<u8>,
}

impl SynapticVoucher {
    pub fn new_signed(
        agent_signing_key: &SigningKey,
        bond_utxo: UtxoId,
        payee_pubkey: [u8; 32],
        sequence: u64,
        cumulative_amount: NanoSkn,
    ) -> Self {
        let payer_agent_pubkey = agent_signing_key.verifying_key().to_bytes();
        let mut v = Self {
            bond_utxo,
            payer_agent_pubkey,
            payee_pubkey,
            sequence,
            cumulative_amount,
            signature: Vec::new(),
        };
        let sig = agent_signing_key.sign(&v.signing_digest());
        v.signature = sig.to_bytes().to_vec();
        v
    }

    pub fn signing_digest(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"SKYNET_SYNAPTIC_VOUCHER_V1");
        h.update(&self.bond_utxo.tx_hash);
        h.update(&self.bond_utxo.vout.to_le_bytes());
        h.update(&self.payer_agent_pubkey);
        h.update(&self.payee_pubkey);
        h.update(&self.sequence.to_le_bytes());
        h.update(&self.cumulative_amount.0.to_le_bytes());
        *h.finalize().as_bytes()
    }

    pub fn verify(&self, max_allowance: NanoSkn) -> Result<(), LedgerError> {
        if self.cumulative_amount.0 < MIN_TX_NANO_SKN {
            return Err(LedgerError::ZeroOutputAmount);
        }
        if self.cumulative_amount > max_allowance {
            return Err(LedgerError::VoucherExceedsAllowance {
                requested: self.cumulative_amount.0,
                allowance: max_allowance.0,
            });
        }
        verify_ed25519(&self.payer_agent_pubkey, &self.signing_digest(), &self.signature)
    }
}

/// Payee-side lightweight tracker absorbing lossy UDP `SynapticVoucher` streams.
#[derive(Debug, Clone)]
pub struct VoucherChannelTracker {
    pub bond_utxo: UtxoId,
    pub max_allowance: NanoSkn,
    pub highest_voucher: Option<SynapticVoucher>,
}

impl VoucherChannelTracker {
    pub fn new(bond_utxo: UtxoId, max_allowance: NanoSkn) -> Self {
        Self {
            bond_utxo,
            max_allowance,
            highest_voucher: None,
        }
    }

    /// Absorbs an incoming UDP voucher, returning incremental `NanoSkn` earned even under packet loss.
    pub fn receive_udp_voucher(&mut self, voucher: SynapticVoucher) -> Result<NanoSkn, LedgerError> {
        if voucher.bond_utxo != self.bond_utxo {
            return Err(LedgerError::UnknownOrSpentUtxo);
        }
        voucher.verify(self.max_allowance)?;

        let Some(prev) = &self.highest_voucher else {
            let earned = voucher.cumulative_amount;
            self.highest_voucher = Some(voucher);
            return Ok(earned);
        };

        if voucher.payer_agent_pubkey != prev.payer_agent_pubkey
            || voucher.payee_pubkey != prev.payee_pubkey
        {
            return Err(LedgerError::UnauthorizedSpend);
        }
        if voucher.sequence <= prev.sequence || voucher.cumulative_amount <= prev.cumulative_amount {
            return Err(LedgerError::StaleVoucher);
        }
        let delta = voucher.cumulative_amount.checked_sub(prev.cumulative_amount)?;
        self.highest_voucher = Some(voucher);
        Ok(delta)
    }

    fn build_settlement_outputs(
        voucher: &SynapticVoucher,
        bond_output: &TxOutput,
        fee: NanoSkn,
    ) -> Result<Vec<TxOutput>, LedgerError> {
        let CustodyPolicy::AgentBond {
            human_owner,
            max_voucher_allowance,
        } = bond_output.policy
        else {
            return Err(LedgerError::UnauthorizedSpend);
        };
        voucher.verify(max_voucher_allowance)?;

        let deducted = voucher.cumulative_amount.checked_add(fee)?;
        let refund_to_human_bond = bond_output.amount.checked_sub(deducted)?;
        let new_remaining_allowance = max_voucher_allowance.checked_sub(voucher.cumulative_amount)?;

        let mut outputs = vec![TxOutput::direct(voucher.payee_pubkey, voucher.cumulative_amount)];
        if refund_to_human_bond.0 > 0 {
            outputs.push(TxOutput::agent_bond(
                human_owner,
                bond_output.owner_pubkey,
                refund_to_human_bond,
                new_remaining_allowance,
            ));
        }
        Ok(outputs)
    }

    /// Builds cooperative settlement transaction returning unspent bonded principal to `human_owner`.
    pub fn build_cooperative_settlement_tx(
        &self,
        payer_agent_key: &SigningKey,
        bond_output: &TxOutput,
        fee: NanoSkn,
        dag_parents: Vec<[u8; 32]>,
        nonce: u64,
    ) -> Result<SknTransaction, LedgerError> {
        let voucher = self
            .highest_voucher
            .as_ref()
            .ok_or(LedgerError::EmptyInputsOrOutputs)?;
        let outputs = Self::build_settlement_outputs(voucher, bond_output, fee)?;
        Ok(SknTransaction::new_signed(
            payer_agent_key,
            vec![self.bond_utxo],
            outputs,
            fee,
            dag_parents,
            nonce,
        ))
    }

    /// Unilateral Payee Settlement authorized by Payer's signed `SynapticVoucher`.
    pub fn build_unilateral_settlement_tx(
        &self,
        payee_key: &SigningKey,
        bond_output: &TxOutput,
        fee: NanoSkn,
        dag_parents: Vec<[u8; 32]>,
        nonce: u64,
    ) -> Result<(SknTransaction, SynapticVoucher), LedgerError> {
        let voucher = self
            .highest_voucher
            .as_ref()
            .ok_or(LedgerError::EmptyInputsOrOutputs)?;
        let outputs = Self::build_settlement_outputs(voucher, bond_output, fee)?;
        let tx = SknTransaction::new_signed(
            payee_key,
            vec![self.bond_utxo],
            outputs,
            fee,
            dag_parents,
            nonce,
        );
        Ok((tx, voucher.clone()))
    }
}
