//! `compute_escrow_refund`: Timeout refunds and full monetary conservation queries for compute escrows.

use crate::compute_escrow::ComputeEscrowVault;
use crate::compute_escrow_types::{EscrowError, EscrowStatus, TaskId};
use crate::NanoSkn;

fn sum_auditor_pairs(base: NanoSkn, pairs: &[([u8; 32], NanoSkn)]) -> Result<NanoSkn, EscrowError> {
    pairs
        .iter()
        .try_fold(base, |acc, (_, b)| acc.checked_add(*b))
        .map_err(Into::into)
}

impl ComputeEscrowVault {
    /// Refunds client upon worker timeout and slashes worker collateral (auditors refunded in full).
    pub fn refund_task(
        &mut self,
        task_id: TaskId,
        client: &[u8; 32],
        current_epoch: u64,
    ) -> Result<(NanoSkn, NanoSkn), EscrowError> {
        let prior = self.chunk_schedules.get(&task_id).map(|s| s.cumulative_released).unwrap_or(NanoSkn::ZERO);
        let record = self.records.get_mut(&task_id).ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked { client_pubkey, bounty, bond, timeout_epoch, auditor_pool, ref auditors, .. } =
            record.status
        else {
            return Err(EscrowError::NotInLockedState);
        };
        if client != &client_pubkey {
            return Err(EscrowError::UnauthorizedClient);
        }
        if current_epoch <= timeout_epoch {
            return Err(EscrowError::NotExpired { current: current_epoch, timeout: timeout_epoch });
        }

        let compensation = NanoSkn(bond.0 / 2);
        let slashed_burn = bond.checked_sub(compensation)?;
        let total_refund = bounty.checked_add(auditor_pool)?.checked_add(compensation)?;
        let auditor_refunds = auditors.iter().map(|a| (a.auditor_pubkey, a.bond)).collect();
        record.status = EscrowStatus::Refunded {
            client_pubkey,
            refund: total_refund,
            slashed_burn: slashed_burn.checked_add(prior)?,
            auditor_refunds,
        };
        Ok((total_refund, slashed_burn))
    }

    /// Verifies exact monetary conservation: locked inputs == disbursed payouts / refunds / slashes.
    pub fn total_accounted_vault_balance(&self, task_id: &TaskId) -> Result<NanoSkn, EscrowError> {
        let record = self.records.get(task_id).ok_or(EscrowError::NotFound)?;
        let prior = self.chunk_schedules.get(task_id).map(|s| s.cumulative_released).unwrap_or(NanoSkn::ZERO);

        match &record.status {
            EscrowStatus::Locked { bounty, bond, auditor_pool, auditors, .. } => {
                let base = bounty.checked_add(*bond)?.checked_add(*auditor_pool)?.checked_add(prior)?;
                auditors.iter().try_fold(base, |acc, a| acc.checked_add(a.bond)).map_err(Into::into)
            }
            EscrowStatus::Completed { payout, .. } => Ok(*payout),
            EscrowStatus::CompletedWithAuditors { breakdown, .. } => Ok(breakdown.total_disbursed),
            EscrowStatus::Refunded { refund, slashed_burn, auditor_refunds, .. } => {
                sum_auditor_pairs(refund.checked_add(*slashed_burn)?, auditor_refunds)
            }
            EscrowStatus::Slashed { client_refund, auditor_reward, prior_chunk_payouts, auditor_refunds, .. } => {
                let base = client_refund.checked_add(*auditor_reward)?.checked_add(*prior_chunk_payouts)?;
                sum_auditor_pairs(base, auditor_refunds)
            }
        }
    }
}
