//! `escrow_slash`: PoUW Canary fraud slashing (`100%` bond seizure) and worker reputation tracking.

use serde::{Deserialize, Serialize};

use crate::compute_escrow::ComputeEscrowVault;
use crate::compute_escrow_types::{ComputeEscrowRecord, EscrowError, EscrowStatus, TaskId};
use crate::NanoSkn;

/// Default initial PoUW reputation score for newly active workers (`50.00%` = `5,000 bps`).
pub const DEFAULT_POUW_SCORE_BPS: u16 = 5_000;
/// Maximum PoUW reputation score ceiling (`100.00%` = `10,000 bps`).
pub const MAX_POUW_SCORE_BPS: u16 = 10_000;

const MIN_MILLI_COSINE: i16 = -1_000;
const MAX_MILLI_COSINE: i16 = 1_000;

/// Cryptographic fraud proof submitted when a Worker fails a deterministic Canary Spot-Check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FraudProofClaim {
    pub task_id: TaskId,
    pub faulty_worker: [u8; 32],
    pub challenger_auditor: [u8; 32],
    pub measured_cosine_milli: i16,
    pub evidence_digest: [u8; 32],
}

/// On-ledger Proof-of-Useful-Work reputation profile for an edge Worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerReputation {
    pub pouw_score_bps: u16,
    pub verified_chunks: u64,
    pub slashed_count: u32,
}

impl Default for WorkerReputation {
    fn default() -> Self {
        Self {
            pouw_score_bps: DEFAULT_POUW_SCORE_BPS,
            verified_chunks: 0,
            slashed_count: 0,
        }
    }
}

/// Settlement receipt produced when a dishonest or broken Worker's bond is slashed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlashOutcome {
    pub task_id: TaskId,
    pub faulty_worker: [u8; 32],
    pub challenger_auditor: [u8; 32],
    pub slashed_bond: NanoSkn,
    pub auditor_reward: NanoSkn,
    pub client_refund: NanoSkn,
    pub auditor_bond_refunds: Vec<([u8; 32], NanoSkn)>,
    pub updated_reputation: WorkerReputation,
}

struct RecordSlashSplit {
    slashed_bond: NanoSkn,
    auditor_reward: NanoSkn,
    client_refund: NanoSkn,
    auditor_bond_refunds: Vec<([u8; 32], NanoSkn)>,
}

fn apply_slash_to_record(
    record: &mut ComputeEscrowRecord,
    claim: FraudProofClaim,
    prior_chunk_payouts: NanoSkn,
) -> Result<RecordSlashSplit, EscrowError> {
    let EscrowStatus::Locked { client_pubkey, worker_pubkey, bounty, bond, auditor_pool, ref auditors, .. } =
        record.status
    else {
        return Err(EscrowError::NotInLockedState);
    };
    if claim.faulty_worker != worker_pubkey {
        return Err(EscrowError::UnauthorizedWorker);
    }
    if claim.challenger_auditor == worker_pubkey {
        return Err(EscrowError::SelfAuditingForbidden);
    }
    let auditor_reward = NanoSkn(bond.0 / 2);
    let client_refund = bounty.checked_add(auditor_pool)?.checked_add(bond.checked_sub(auditor_reward)?)?;
    let auditor_bond_refunds: Vec<([u8; 32], NanoSkn)> = auditors.iter().map(|a| (a.auditor_pubkey, a.bond)).collect();
    record.status = EscrowStatus::Slashed {
        client_pubkey,
        faulty_worker: worker_pubkey,
        challenger_auditor: claim.challenger_auditor,
        client_refund,
        auditor_reward,
        prior_chunk_payouts,
        auditor_refunds: auditor_bond_refunds.clone(),
        evidence_digest: claim.evidence_digest,
    };
    Ok(RecordSlashSplit { slashed_bond: bond, auditor_reward, client_refund, auditor_bond_refunds })
}

impl ComputeEscrowVault {
    /// Returns the current `WorkerReputation` for `worker_pubkey` (defaults to `5,000 bps`).
    pub fn worker_reputation(&self, worker_pubkey: &[u8; 32]) -> WorkerReputation {
        self.reputations.get(worker_pubkey).copied().unwrap_or_default()
    }

    /// Credits verified PoUW chunk progress to `worker_pubkey` capped at `MAX_POUW_SCORE_BPS`.
    pub fn record_verified_chunk_reputation(&mut self, worker_pubkey: [u8; 32], gain_bps: u16) -> WorkerReputation {
        let rep = self.reputations.entry(worker_pubkey).or_default();
        rep.verified_chunks = rep.verified_chunks.saturating_add(1);
        rep.pouw_score_bps = rep.pouw_score_bps.saturating_add(gain_bps).min(MAX_POUW_SCORE_BPS);
        *rep
    }

    /// Slashes `100%` of faulty Worker's bond (`50%` challenger reward + `100%` bounty & `50%` bond to Client)
    /// and zeroes the Worker's `pouw_score_bps`.
    pub fn slash_on_fraud_proof(&mut self, claim: FraudProofClaim) -> Result<SlashOutcome, EscrowError> {
        if !(MIN_MILLI_COSINE..=MAX_MILLI_COSINE).contains(&claim.measured_cosine_milli) {
            return Err(EscrowError::InvalidFraudProof);
        }
        let prior = self.chunk_schedules.get(&claim.task_id).map(|s| s.cumulative_released).unwrap_or(NanoSkn::ZERO);
        let record = self.records.get_mut(&claim.task_id).ok_or(EscrowError::NotFound)?;
        let split = apply_slash_to_record(record, claim, prior)?;

        let rep = self.reputations.entry(claim.faulty_worker).or_default();
        rep.pouw_score_bps = 0;
        rep.slashed_count = rep.slashed_count.saturating_add(1);
        let updated_reputation = *rep;

        Ok(SlashOutcome {
            task_id: claim.task_id,
            faulty_worker: claim.faulty_worker,
            challenger_auditor: claim.challenger_auditor,
            slashed_bond: split.slashed_bond,
            auditor_reward: split.auditor_reward,
            client_refund: split.client_refund,
            auditor_bond_refunds: split.auditor_bond_refunds,
            updated_reputation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EscrowSplitConfig;

    #[test]
    fn fraud_proof_slashes_full_bond_zeros_reputation_and_conserves_odd_nanoskn() {
        let mut vault = ComputeEscrowVault::new();
        let task_id = [0x42; 32];
        let (client, worker, auditor) = ([1u8; 32], [2u8; 32], [3u8; 32]);
        let (bounty, odd_bond) = (NanoSkn::from_nano(1_000), NanoSkn::from_nano(301));

        vault.lock_task_simple(task_id, client, worker, bounty, odd_bond, 100).unwrap();
        let locked_before = vault.total_accounted_vault_balance(&task_id).unwrap();
        vault.record_verified_chunk_reputation(worker, 500);
        assert_eq!(vault.worker_reputation(&worker).pouw_score_bps, 5_500);

        let claim = FraudProofClaim {
            task_id,
            faulty_worker: worker,
            challenger_auditor: auditor,
            measured_cosine_milli: 410,
            evidence_digest: [0xEE; 32],
        };
        let outcome = vault.slash_on_fraud_proof(claim).unwrap();
        assert_eq!((outcome.slashed_bond, outcome.auditor_reward, outcome.client_refund), (odd_bond, NanoSkn(150), NanoSkn(1_151)));
        assert_eq!((outcome.updated_reputation.pouw_score_bps, outcome.updated_reputation.slashed_count), (0, 1));
        assert_eq!(vault.total_accounted_vault_balance(&task_id).unwrap(), locked_before);
    }

    #[test]
    fn fraud_slash_blocks_self_challenge_and_refunds_registered_auditors() {
        let mut vault = ComputeEscrowVault::new();
        let task_id = [0x55; 32];
        let (client, worker, aud1) = ([1u8; 32], [2u8; 32], [9u8; 32]);
        let split = EscrowSplitConfig { worker_bounty: NanoSkn(800), auditor_pool_bounty: NanoSkn(200), required_auditors: 1 };
        vault.lock_task_with_split(task_id, client, worker, split, NanoSkn(400), 90).unwrap();
        vault.register_auditor(task_id, aud1, NanoSkn(75)).unwrap();
        let before = vault.total_accounted_vault_balance(&task_id).unwrap();

        let self_claim = FraudProofClaim { task_id, faulty_worker: worker, challenger_auditor: worker, measured_cosine_milli: -100, evidence_digest: [1u8; 32] };
        assert_eq!(vault.slash_on_fraud_proof(self_claim), Err(EscrowError::SelfAuditingForbidden));

        let out = vault.slash_on_fraud_proof(FraudProofClaim { challenger_auditor: aud1, ..self_claim }).unwrap();
        assert_eq!((out.auditor_reward, out.client_refund), (NanoSkn(200), NanoSkn(1_200)));
        assert_eq!(out.auditor_bond_refunds, vec![(aud1, NanoSkn(75))]);
        assert_eq!(vault.total_accounted_vault_balance(&task_id).unwrap(), before);
    }
}
