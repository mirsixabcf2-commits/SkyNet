//! `hwid_airdrop`: Zero-inflation hardware-attested bootstrap marketing vault
//! and biological agent bond distribution for SkyNet early adoption.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{CustodyPolicy, LedgerError, NanoSkn, TxOutput, UtxoId};

pub type HwidDigest = [u8; 32];
pub type ProofDigest = [u8; 32];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MarketingError {
    #[error("hardware identifier already claimed an airdrop")]
    HwidAlreadyClaimed,
    #[error("marketing bootstrap reserve is depleted")]
    ReserveExhausted,
    #[error("zero reward calculated for claim")]
    ZeroReward,
    #[error("invalid hardware proof or unverified signature")]
    InvalidProof,
    #[error("ledger core error: {0}")]
    Ledger(#[from] LedgerError),
}

/// Registration record for an attested hardware device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HwidClaimRecord {
    pub hwid_digest: HwidDigest,
    pub claimed_epoch: u64,
    pub allocated_bond: NanoSkn,
    pub utxo_id: UtxoId,
    pub proof_digest: ProofDigest,
}

/// Request parameters for claiming an attested hardware bootstrap airdrop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HwidClaimRequest {
    pub hwid: HwidDigest,
    pub human_owner: [u8; 32],
    pub agent_pubkey: [u8; 32],
    pub epoch: u64,
    pub proof_digest: ProofDigest,
    pub tx_hash: [u8; 32],
    pub vout: u16,
    pub attestation_sig: Vec<u8>,
}

impl HwidClaimRequest {
    pub fn new_simple(
        hwid: HwidDigest,
        human_owner: [u8; 32],
        agent_pubkey: [u8; 32],
        epoch: u64,
        proof_digest: ProofDigest,
        tx_hash: [u8; 32],
        vout: u16,
    ) -> Self {
        Self {
            hwid,
            human_owner,
            agent_pubkey,
            epoch,
            proof_digest,
            tx_hash,
            vout,
            attestation_sig: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_attested(
        hwid: HwidDigest,
        human_owner: [u8; 32],
        agent_pubkey: [u8; 32],
        epoch: u64,
        proof_digest: ProofDigest,
        tx_hash: [u8; 32],
        vout: u16,
        attestation_sig: Vec<u8>,
    ) -> Self {
        Self {
            hwid,
            human_owner,
            agent_pubkey,
            epoch,
            proof_digest,
            tx_hash,
            vout,
            attestation_sig,
        }
    }

    pub fn signing_digest(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"SKYNET_HWID_CLAIM_ATTEST_V1");
        h.update(&self.hwid);
        h.update(&self.human_owner);
        h.update(&self.agent_pubkey);
        h.update(&self.epoch.to_le_bytes());
        h.update(&self.proof_digest);
        *h.finalize().as_bytes()
    }

    pub fn verify_signature(&self) -> Result<(), MarketingError> {
        use ed25519_dalek::{Signature, VerifyingKey};
        let vk = VerifyingKey::from_bytes(&self.agent_pubkey)
            .map_err(|_| MarketingError::InvalidProof)?;
        let sig_arr: [u8; 64] = self
            .attestation_sig
            .as_slice()
            .try_into()
            .map_err(|_| MarketingError::InvalidProof)?;
        let sig = Signature::from_bytes(&sig_arr);
        vk.verify_strict(&self.signing_digest(), &sig)
            .map_err(|_| MarketingError::InvalidProof)
    }
}

/// Vault managing fixed Genesis marketing allocations with halving decay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketingBootstrapVault {
    pub total_allocated: NanoSkn,
    pub remaining_reserve: NanoSkn,
    pub total_devices_registered: u64,
    pub base_reward: NanoSkn,
    pub halving_step_devices: u64,
    claimed_devices: HashMap<HwidDigest, HwidClaimRecord>,
}

impl MarketingBootstrapVault {
    /// Creates a marketing vault backed by an allocated Genesis reserve.
    pub fn new(reserve: NanoSkn, base_reward: NanoSkn, halving_step_devices: u64) -> Self {
        Self {
            total_allocated: NanoSkn::ZERO,
            remaining_reserve: reserve,
            total_devices_registered: 0,
            base_reward,
            halving_step_devices: halving_step_devices.max(1),
            claimed_devices: HashMap::new(),
        }
    }

    /// Computes current claim reward under the progressive halving schedule.
    pub fn calculate_current_reward(&self) -> NanoSkn {
        let shift = self.total_devices_registered / self.halving_step_devices;
        if shift >= 64 {
            return NanoSkn::ZERO;
        }
        NanoSkn(self.base_reward.0 >> shift)
    }

    pub fn is_claimed(&self, hwid: &HwidDigest) -> bool {
        self.claimed_devices.contains_key(hwid)
    }

    pub fn get_claim(&self, hwid: &HwidDigest) -> Option<&HwidClaimRecord> {
        self.claimed_devices.get(hwid)
    }

    pub fn total_accounted_reserve(&self) -> Result<NanoSkn, LedgerError> {
        self.total_allocated.checked_add(self.remaining_reserve)
    }

    /// Allocates bonded SKN to a verified hardware node.
    pub fn claim_reward(
        &mut self,
        req: HwidClaimRequest,
    ) -> Result<(NanoSkn, TxOutput), MarketingError> {
        if self.is_claimed(&req.hwid) {
            return Err(MarketingError::HwidAlreadyClaimed);
        }
        if req.hwid == [0u8; 32] || req.proof_digest == [0u8; 32] {
            return Err(MarketingError::InvalidProof);
        }
        if !req.attestation_sig.is_empty() {
            req.verify_signature()?;
        }
        let reward = self.calculate_current_reward();
        if reward.0 == 0 || self.remaining_reserve.0 == 0 {
            return Err(MarketingError::ReserveExhausted);
        }

        let grant = NanoSkn(reward.0.min(self.remaining_reserve.0));
        self.remaining_reserve = self.remaining_reserve.checked_sub(grant)?;
        self.total_allocated = self.total_allocated.checked_add(grant)?;
        self.total_devices_registered = self.total_devices_registered.saturating_add(1);

        let utxo_id = UtxoId {
            tx_hash: req.tx_hash,
            vout: req.vout,
        };
        self.claimed_devices.insert(
            req.hwid,
            HwidClaimRecord {
                hwid_digest: req.hwid,
                claimed_epoch: req.epoch,
                allocated_bond: grant,
                utxo_id,
                proof_digest: req.proof_digest,
            },
        );

        let output = TxOutput {
            owner_pubkey: req.agent_pubkey,
            amount: grant,
            policy: CustodyPolicy::AgentBond {
                human_owner: req.human_owner,
                max_voucher_allowance: NanoSkn::ZERO,
            },
        };
        Ok((grant, output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim_reward_success_and_conservation() {
        let reserve = NanoSkn::from_skn(100).unwrap();
        let base_reward = NanoSkn::from_skn(1).unwrap();
        let mut vault = MarketingBootstrapVault::new(reserve, base_reward, 10);

        let req = HwidClaimRequest::new_simple(
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
            1,
            [4u8; 32],
            [5u8; 32],
            0,
        );

        let (grant, output) = vault.claim_reward(req).expect("claim must succeed");

        assert_eq!(grant, base_reward);
        assert_eq!(output.amount, base_reward);
        assert_eq!(output.owner_pubkey, [3u8; 32]);
        assert_eq!(
            output.policy,
            CustodyPolicy::AgentBond {
                human_owner: [2u8; 32],
                max_voucher_allowance: NanoSkn::ZERO,
            }
        );
        assert_eq!(vault.total_devices_registered, 1);
        assert_eq!(vault.total_accounted_reserve().unwrap(), reserve);
    }

    #[test]
    fn test_duplicate_claim_rejected() {
        let reserve = NanoSkn::from_skn(100).unwrap();
        let base_reward = NanoSkn::from_skn(1).unwrap();
        let mut vault = MarketingBootstrapVault::new(reserve, base_reward, 10);
        let hwid = [9u8; 32];

        vault
            .claim_reward(HwidClaimRequest::new_simple(
                hwid,
                [1u8; 32],
                [2u8; 32],
                1,
                [0xAAu8; 32],
                [7u8; 32],
                0,
            ))
            .unwrap();

        let err = vault
            .claim_reward(HwidClaimRequest::new_simple(
                hwid,
                [1u8; 32],
                [2u8; 32],
                2,
                [0xAAu8; 32],
                [8u8; 32],
                0,
            ))
            .unwrap_err();
        assert_eq!(err, MarketingError::HwidAlreadyClaimed);
    }

    #[test]
    fn test_halving_progression() {
        let reserve = NanoSkn::from_skn(100).unwrap();
        let base_reward = NanoSkn::from_skn(4).unwrap();
        let mut vault = MarketingBootstrapVault::new(reserve, base_reward, 2);

        // Tier 0 (0-1 devices): 4 SKN
        assert_eq!(vault.calculate_current_reward(), base_reward);
        vault
            .claim_reward(HwidClaimRequest::new_simple(
                [1u8; 32],
                [0u8; 32],
                [0u8; 32],
                1,
                [0xAAu8; 32],
                [0u8; 32],
                0,
            ))
            .unwrap();
        vault
            .claim_reward(HwidClaimRequest::new_simple(
                [2u8; 32],
                [0u8; 32],
                [0u8; 32],
                1,
                [0xAAu8; 32],
                [0u8; 32],
                1,
            ))
            .unwrap();

        // Tier 1 (2-3 devices): 2 SKN
        assert_eq!(
            vault.calculate_current_reward(),
            NanoSkn::from_skn(2).unwrap()
        );
        vault
            .claim_reward(HwidClaimRequest::new_simple(
                [3u8; 32],
                [0u8; 32],
                [0u8; 32],
                1,
                [0xAAu8; 32],
                [0u8; 32],
                2,
            ))
            .unwrap();

        // Tier 2 (after 4 devices): 1 SKN
        vault
            .claim_reward(HwidClaimRequest::new_simple(
                [4u8; 32],
                [0u8; 32],
                [0u8; 32],
                1,
                [0xAAu8; 32],
                [0u8; 32],
                3,
            ))
            .unwrap();
        assert_eq!(
            vault.calculate_current_reward(),
            NanoSkn::from_skn(1).unwrap()
        );
    }

    #[test]
    fn test_reserve_exhaustion() {
        let reserve = NanoSkn::from_skn(1).unwrap();
        let base_reward = NanoSkn::from_skn(1).unwrap();
        let mut vault = MarketingBootstrapVault::new(reserve, base_reward, 100);

        vault
            .claim_reward(HwidClaimRequest::new_simple(
                [1u8; 32],
                [0u8; 32],
                [0u8; 32],
                1,
                [0xAAu8; 32],
                [0u8; 32],
                0,
            ))
            .unwrap();

        let err = vault
            .claim_reward(HwidClaimRequest::new_simple(
                [2u8; 32],
                [0u8; 32],
                [0u8; 32],
                1,
                [0xAAu8; 32],
                [0u8; 32],
                1,
            ))
            .unwrap_err();
        assert_eq!(err, MarketingError::ReserveExhausted);
    }

    #[test]
    fn test_claim_reward_attested_signature_and_tamper_detection() {
        use ed25519_dalek::{Signer, SigningKey};
        use rand::rngs::OsRng;

        let signing_key = SigningKey::generate(&mut OsRng);
        let agent_pubkey = signing_key.verifying_key().to_bytes();

        let reserve = NanoSkn::from_skn(100).unwrap();
        let base_reward = NanoSkn::from_skn(1).unwrap();
        let mut vault = MarketingBootstrapVault::new(reserve, base_reward, 10);

        let hwid = [0x55u8; 32];
        let human_owner = [0x77u8; 32];
        let epoch = 10;
        let proof_digest = [0x99u8; 32];

        let mut req = HwidClaimRequest::new_simple(
            hwid,
            human_owner,
            agent_pubkey,
            epoch,
            proof_digest,
            [1u8; 32],
            0,
        );
        let sig = signing_key.sign(&req.signing_digest());
        req.attestation_sig = sig.to_bytes().to_vec();

        // 1. Valid signature succeeds
        let (grant, out) = vault.claim_reward(req.clone()).expect("attested claim must succeed");
        assert_eq!(grant, base_reward);
        assert_eq!(out.owner_pubkey, agent_pubkey);

        // 2. Tampered signature is rejected
        let mut tampered_req = HwidClaimRequest::new_simple(
            [0x66u8; 32], // different HWID
            human_owner,
            agent_pubkey,
            epoch,
            proof_digest,
            [2u8; 32],
            0,
        );
        tampered_req.attestation_sig = vec![0u8; 64]; // invalid sig
        let err = vault.claim_reward(tampered_req).unwrap_err();
        assert_eq!(err, MarketingError::InvalidProof);

        // 3. Zero proof digest is rejected
        let zero_proof_req = HwidClaimRequest::new_simple(
            [0x88u8; 32],
            human_owner,
            agent_pubkey,
            epoch,
            [0u8; 32], // zero proof
            [3u8; 32],
            0,
        );
        let err2 = vault.claim_reward(zero_proof_req).unwrap_err();
        assert_eq!(err2, MarketingError::InvalidProof);
    }
}
