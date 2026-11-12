//! `custody`: Soul-Body Custody covenants, UTXO outputs, Human Sovereign Wallet, and DAG transactions.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::{LedgerError, NanoSkn};

pub(crate) fn verify_ed25519(pubkey: &[u8; 32], digest: &[u8; 32], sig_bytes: &[u8]) -> Result<(), LedgerError> {
    let vk = VerifyingKey::from_bytes(pubkey).map_err(|_| LedgerError::InvalidSignature)?;
    let sig_arr: [u8; 64] = sig_bytes.try_into().map_err(|_| LedgerError::InvalidSignature)?;
    vk.verify(digest, &Signature::from_bytes(&sig_arr)).map_err(|_| LedgerError::InvalidSignature)
}

/// Unique identifier of an unspent SKN output in the DAG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UtxoId {
    pub tx_hash: [u8; 32],
    pub vout: u16,
}

/// Custody covenant separating **Human Sovereign Cold Ownership** from **AI Agent Hot Operation**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyPolicy {
    DirectOwner,
    AgentBond { human_owner: [u8; 32], max_voucher_allowance: NanoSkn },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOutput {
    pub owner_pubkey: [u8; 32],
    pub amount: NanoSkn,
    pub policy: CustodyPolicy,
}

impl TxOutput {
    pub fn direct(owner_pubkey: [u8; 32], amount: NanoSkn) -> Self {
        Self { owner_pubkey, amount, policy: CustodyPolicy::DirectOwner }
    }

    pub fn agent_bond(
        human_owner: [u8; 32],
        agent_pubkey: [u8; 32],
        amount: NanoSkn,
        max_voucher_allowance: NanoSkn,
    ) -> Self {
        let policy = CustodyPolicy::AgentBond { human_owner, max_voucher_allowance };
        Self { owner_pubkey: agent_pubkey, amount, policy }
    }

    pub fn human_beneficiary(&self) -> [u8; 32] {
        match &self.policy {
            CustodyPolicy::DirectOwner => self.owner_pubkey,
            CustodyPolicy::AgentBond { human_owner, .. } => *human_owner,
        }
    }

    pub(crate) fn hash_into(&self, hasher: &mut blake3::Hasher) {
        hasher.update(&self.owner_pubkey);
        hasher.update(&self.amount.0.to_le_bytes());
        match &self.policy {
            CustodyPolicy::DirectOwner => {
                hasher.update(&[0u8]);
            }
            CustodyPolicy::AgentBond { human_owner, max_voucher_allowance } => {
                hasher.update(&[1u8]);
                hasher.update(human_owner);
                hasher.update(&max_voucher_allowance.0.to_le_bytes());
            }
        }
    }
}

/// Signed offline certificate issued by a Human Wallet authorizing an `AgentNode` hot key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentOwnershipCertificate {
    pub human_owner_pubkey: [u8; 32],
    pub agent_pubkey: [u8; 32],
    pub agent_index: u32,
    pub max_voucher_allowance: NanoSkn,
    pub valid_until_epoch: u64,
    pub human_signature: Vec<u8>,
}

impl AgentOwnershipCertificate {
    pub fn signing_digest(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"SKYNET_HUMAN_AGENT_CERT_V1");
        h.update(&self.human_owner_pubkey);
        h.update(&self.agent_pubkey);
        h.update(&self.agent_index.to_le_bytes());
        h.update(&self.max_voucher_allowance.0.to_le_bytes());
        h.update(&self.valid_until_epoch.to_le_bytes());
        *h.finalize().as_bytes()
    }

    pub fn verify(&self, current_epoch: u64) -> Result<(), LedgerError> {
        if current_epoch > self.valid_until_epoch {
            return Err(LedgerError::UnauthorizedSpend);
        }
        verify_ed25519(&self.human_owner_pubkey, &self.signing_digest(), &self.human_signature)
    }
}

/// Parameters for funding an AI Agent from a Human's cold UTXO.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentFundingParams {
    pub cold_utxo_id: UtxoId,
    pub cold_utxo_amount: NanoSkn,
    pub agent_pubkey: [u8; 32],
    pub bond_amount: NanoSkn,
    pub max_voucher_allowance: NanoSkn,
    pub fee: NanoSkn,
    pub dag_parents: Vec<[u8; 32]>,
    pub nonce: u64,
}

/// Deterministic Hierarchical Human Wallet (`BLAKE3` KDF) isolating cold seed from Agent hot keys.
pub struct HumanWallet {
    master_seed: [u8; 32],
    master_signing_key: SigningKey,
}

impl HumanWallet {
    pub fn from_recovery_phrase(phrase: &str) -> Self {
        Self::from_master_seed(blake3::derive_key("skynet/wallet/master-seed/v1", phrase.as_bytes()))
    }

    pub fn from_master_seed(master_seed: [u8; 32]) -> Self {
        let key_bytes = blake3::derive_key("skynet/wallet/human-cold-ed25519/v1", &master_seed);
        Self { master_seed, master_signing_key: SigningKey::from_bytes(&key_bytes) }
    }

    pub fn human_pubkey(&self) -> [u8; 32] {
        self.master_signing_key.verifying_key().to_bytes()
    }

    pub fn master_signing_key(&self) -> &SigningKey {
        &self.master_signing_key
    }

    pub fn derive_agent_hotkey(&self, agent_index: u32) -> SigningKey {
        let mut ctx = Vec::with_capacity(36);
        ctx.extend_from_slice(&self.master_seed);
        ctx.extend_from_slice(&agent_index.to_le_bytes());
        SigningKey::from_bytes(&blake3::derive_key("skynet/wallet/agent-hotkey-ed25519/v1", &ctx))
    }

    pub fn authorize_agent(&self, idx: u32, allowance: NanoSkn, valid_until: u64) -> (SigningKey, AgentOwnershipCertificate) {
        let agent_key = self.derive_agent_hotkey(idx);
        let mut cert = AgentOwnershipCertificate {
            human_owner_pubkey: self.human_pubkey(),
            agent_pubkey: agent_key.verifying_key().to_bytes(),
            agent_index: idx,
            max_voucher_allowance: allowance,
            valid_until_epoch: valid_until,
            human_signature: Vec::new(),
        };
        cert.human_signature = self.master_signing_key.sign(&cert.signing_digest()).to_bytes().to_vec();
        (agent_key, cert)
    }

    pub fn create_agent_funding_tx(&self, p: AgentFundingParams) -> Result<SknTransaction, LedgerError> {
        let change = p.cold_utxo_amount.checked_sub(p.bond_amount.checked_add(p.fee)?)?;
        let mut outs = vec![TxOutput::agent_bond(self.human_pubkey(), p.agent_pubkey, p.bond_amount, p.max_voucher_allowance)];
        if change.0 > 0 {
            outs.push(TxOutput::direct(self.human_pubkey(), change));
        }
        Ok(SknTransaction::new_signed(&self.master_signing_key, vec![p.cold_utxo_id], outs, p.fee, p.dag_parents, p.nonce))
    }

    pub fn build_sovereign_reclaim_tx(
        &self,
        bond_utxo_id: UtxoId,
        bond_amount: NanoSkn,
        fee: NanoSkn,
        dag_parents: Vec<[u8; 32]>,
        nonce: u64,
    ) -> Result<SknTransaction, LedgerError> {
        let outs = vec![TxOutput::direct(self.human_pubkey(), bond_amount.checked_sub(fee)?)];
        Ok(SknTransaction::new_signed(&self.master_signing_key, vec![bond_utxo_id], outs, fee, dag_parents, nonce))
    }
}

/// Signed SKN UTXO-DAG transaction (`<= 1200B` UDP datagram friendly).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SknTransaction {
    pub inputs: Vec<UtxoId>,
    pub outputs: Vec<TxOutput>,
    pub fee: NanoSkn,
    pub dag_parents: Vec<[u8; 32]>,
    pub nonce: u64,
    pub sender_pubkey: [u8; 32],
    pub signature_bytes: Vec<u8>,
}

impl SknTransaction {
    pub fn new_signed(
        key: &SigningKey,
        inputs: Vec<UtxoId>,
        outputs: Vec<TxOutput>,
        fee: NanoSkn,
        dag_parents: Vec<[u8; 32]>,
        nonce: u64,
    ) -> Self {
        let sender_pubkey = key.verifying_key().to_bytes();
        let mut tx = Self { inputs, outputs, fee, dag_parents, nonce, sender_pubkey, signature_bytes: Vec::new() };
        tx.signature_bytes = key.sign(&tx.signing_digest()).to_bytes().to_vec();
        tx
    }

    pub fn signing_digest(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"SKYNET_TX_V2");
        h.update(&(self.inputs.len() as u32).to_le_bytes());
        for inp in &self.inputs {
            h.update(&inp.tx_hash);
            h.update(&inp.vout.to_le_bytes());
        }
        h.update(&(self.outputs.len() as u32).to_le_bytes());
        for out in &self.outputs {
            out.hash_into(&mut h);
        }
        h.update(&self.fee.0.to_le_bytes());
        h.update(&(self.dag_parents.len() as u32).to_le_bytes());
        for p in &self.dag_parents {
            h.update(p);
        }
        h.update(&self.nonce.to_le_bytes());
        h.update(&self.sender_pubkey);
        *h.finalize().as_bytes()
    }

    pub fn tx_hash(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(&self.signing_digest());
        h.update(&self.signature_bytes);
        *h.finalize().as_bytes()
    }

    pub fn verify_signature(&self) -> Result<(), LedgerError> {
        verify_ed25519(&self.sender_pubkey, &self.signing_digest(), &self.signature_bytes)
    }
}
