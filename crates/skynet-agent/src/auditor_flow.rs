//! `auditor_flow`: Multi-party delegated auditor bidding, cryptographic attestation,
//! and quorum settlement coordination for resource-constrained client devices.

use std::collections::HashSet;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use skynet_ledger::compute_escrow::ComputeEscrowVault;
use skynet_ledger::{EscrowPayoutBreakdown, NanoSkn};
use skynet_wire::{ComputeAuditorAttestation, ComputeAuditorBid, TaskId};

use crate::compute_flow::ComputeFlowError;

/// Coordinator managing the lifecycle of delegated community verifiers (Auditors).
pub struct AuditorCoordinator;

impl AuditorCoordinator {
    /// Auditor constructs a signed bid staking bond to participate in verification.
    pub fn create_bid(
        auditor_key: &SigningKey,
        task_id: TaskId,
        bond: NanoSkn,
    ) -> ComputeAuditorBid {
        let mut payload = Vec::with_capacity(72);
        payload.extend_from_slice(b"SKYNET_AUDITOR_BID_V1");
        payload.extend_from_slice(&task_id);
        payload.extend_from_slice(&bond.0.to_le_bytes());
        let sig = auditor_key.sign(&payload);

        ComputeAuditorBid {
            task_id,
            auditor_pubkey: auditor_key.verifying_key().to_bytes(),
            bond_nano_skn: bond.0,
            auditor_signature: sig.to_bytes().to_vec(),
        }
    }

    /// Registers the auditor's staked bond inside the task escrow vault after verifying signature.
    pub fn register_bid(
        vault: &mut ComputeEscrowVault,
        bid: &ComputeAuditorBid,
    ) -> Result<(), ComputeFlowError> {
        let vk = VerifyingKey::from_bytes(&bid.auditor_pubkey)
            .map_err(|_| ComputeFlowError::InvalidSignature)?;
        let sig_arr: [u8; 64] = bid
            .auditor_signature
            .as_slice()
            .try_into()
            .map_err(|_| ComputeFlowError::InvalidSignature)?;
        let sig = Signature::from_bytes(&sig_arr);

        let mut payload = Vec::with_capacity(72);
        payload.extend_from_slice(b"SKYNET_AUDITOR_BID_V1");
        payload.extend_from_slice(&bid.task_id);
        payload.extend_from_slice(&bid.bond_nano_skn.to_le_bytes());
        vk.verify(&payload, &sig)
            .map_err(|_| ComputeFlowError::InvalidSignature)?;

        vault
            .register_auditor(
                bid.task_id,
                bid.auditor_pubkey,
                NanoSkn::from_nano(bid.bond_nano_skn),
            )
            .map_err(ComputeFlowError::Escrow)
    }

    /// Auditor audits result (checking output correctness) and issues signed attestation.
    pub fn create_attestation(
        auditor_key: &SigningKey,
        task_id: TaskId,
        result_digest: [u8; 32],
    ) -> ComputeAuditorAttestation {
        let mut payload = Vec::with_capacity(96);
        payload.extend_from_slice(b"SKYNET_AUDITOR_ATTESTATION_V1");
        payload.extend_from_slice(&task_id);
        payload.extend_from_slice(&result_digest);
        let sig = auditor_key.sign(&payload);

        ComputeAuditorAttestation {
            task_id,
            auditor_pubkey: auditor_key.verifying_key().to_bytes(),
            result_digest,
            auditor_signature: sig.to_bytes().to_vec(),
        }
    }

    /// Verifies that an auditor's attestation is cryptographically authentic.
    pub fn verify_attestation(
        attestation: &ComputeAuditorAttestation,
    ) -> Result<(), ComputeFlowError> {
        let vk = VerifyingKey::from_bytes(&attestation.auditor_pubkey)
            .map_err(|_| ComputeFlowError::InvalidSignature)?;
        let sig_arr: [u8; 64] = attestation
            .auditor_signature
            .as_slice()
            .try_into()
            .map_err(|_| ComputeFlowError::InvalidSignature)?;
        let sig = Signature::from_bytes(&sig_arr);

        let mut payload = Vec::with_capacity(96);
        payload.extend_from_slice(b"SKYNET_AUDITOR_ATTESTATION_V1");
        payload.extend_from_slice(&attestation.task_id);
        payload.extend_from_slice(&attestation.result_digest);
        vk.verify(&payload, &sig)
            .map_err(|_| ComputeFlowError::InvalidSignature)?;
        Ok(())
    }

    /// Settles the task escrow with auditor quorum, distributing reward pool to workers and auditors.
    pub fn settle_with_quorum(
        vault: &mut ComputeEscrowVault,
        task_id: TaskId,
        worker_pubkey: &[u8; 32],
        attestations: &[ComputeAuditorAttestation],
        result_digest: [u8; 32],
        current_epoch: u64,
    ) -> Result<EscrowPayoutBreakdown, ComputeFlowError> {
        let mut seen = HashSet::with_capacity(attestations.len());
        let mut attesting_keys = Vec::with_capacity(attestations.len());
        for att in attestations {
            if att.task_id != task_id {
                return Err(ComputeFlowError::TaskIdMismatch);
            }
            if att.result_digest != result_digest {
                return Err(ComputeFlowError::DigestMismatch);
            }
            if !seen.insert(att.auditor_pubkey) {
                return Err(ComputeFlowError::DuplicateAuditor);
            }
            Self::verify_attestation(att)?;
            attesting_keys.push(att.auditor_pubkey);
        }

        let breakdown = vault
            .complete_task_with_auditors(
                task_id,
                worker_pubkey,
                &attesting_keys,
                result_digest,
                current_epoch,
            )
            .map_err(ComputeFlowError::Escrow)?;

        Ok(breakdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;
    use skynet_ledger::EscrowSplitConfig;

    #[test]
    fn test_auditor_quorum_settlement_flow() {
        let mut vault = ComputeEscrowVault::new();
        let mut csprng = OsRng;

        let client_key = SigningKey::generate(&mut csprng);
        let worker_key = SigningKey::generate(&mut csprng);
        let aud1_key = SigningKey::generate(&mut csprng);
        let aud2_key = SigningKey::generate(&mut csprng);

        let task_id = [77u8; 32];
        let split = EscrowSplitConfig {
            worker_bounty: NanoSkn::from_nano(850),
            auditor_pool_bounty: NanoSkn::from_nano(150),
            required_auditors: 2,
        };
        let worker_bond = NanoSkn::from_nano(200);

        vault
            .lock_task_with_split(
                task_id,
                client_key.verifying_key().to_bytes(),
                worker_key.verifying_key().to_bytes(),
                split,
                worker_bond,
                100,
            )
            .expect("lock split succeeds");

        let bid1 = AuditorCoordinator::create_bid(&aud1_key, task_id, NanoSkn::from_nano(50));
        let bid2 = AuditorCoordinator::create_bid(&aud2_key, task_id, NanoSkn::from_nano(50));

        AuditorCoordinator::register_bid(&mut vault, &bid1).expect("register bid1 succeeds");
        AuditorCoordinator::register_bid(&mut vault, &bid2).expect("register bid2 succeeds");

        let result_digest = [33u8; 32];
        let att1 = AuditorCoordinator::create_attestation(&aud1_key, task_id, result_digest);
        let att2 = AuditorCoordinator::create_attestation(&aud2_key, task_id, result_digest);

        let attestations = vec![att1, att2];
        let breakdown = AuditorCoordinator::settle_with_quorum(
            &mut vault,
            task_id,
            &worker_key.verifying_key().to_bytes(),
            &attestations,
            result_digest,
            50,
        )
        .expect("settle quorum succeeds");

        assert_eq!(breakdown.worker_payout, NanoSkn::from_nano(1050));
        assert_eq!(breakdown.auditor_payouts.len(), 2);
        assert_eq!(breakdown.auditor_payouts[0].1, NanoSkn::from_nano(125)); // 50 bond + 75 bounty
        assert_eq!(breakdown.auditor_payouts[1].1, NanoSkn::from_nano(125));
        assert_eq!(breakdown.total_disbursed, NanoSkn::from_nano(1300));
    }

    #[test]
    fn test_duplicate_auditor_attestation_rejected() {
        let mut vault = ComputeEscrowVault::new();
        let mut csprng = OsRng;

        let aud1_key = SigningKey::generate(&mut csprng);
        let worker_key = SigningKey::generate(&mut csprng);
        let task_id = [78u8; 32];
        let result_digest = [34u8; 32];

        let att = AuditorCoordinator::create_attestation(&aud1_key, task_id, result_digest);
        let attestations = vec![att.clone(), att];

        let err = AuditorCoordinator::settle_with_quorum(
            &mut vault,
            task_id,
            &worker_key.verifying_key().to_bytes(),
            &attestations,
            result_digest,
            50,
        )
        .unwrap_err();
        assert!(matches!(err, ComputeFlowError::DuplicateAuditor));
    }
}
