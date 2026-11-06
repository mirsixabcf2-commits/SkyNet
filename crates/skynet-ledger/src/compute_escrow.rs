//! `compute_escrow`: Two-sided economic bounty escrow (`NanoSkn`), auditor quorum splitting,
//! and collateral slashing for verifiable remote compute tasks on the SkyNet ledger.

use std::collections::HashMap;

use crate::compute_escrow_settle::calculate_audited_payout;
use crate::compute_escrow_types::{
    AuditorDeposit, ComputeEscrowRecord, EscrowError, EscrowPayoutBreakdown, EscrowSplitConfig,
    EscrowStatus, LockTaskParams, TaskId,
};
use crate::NanoSkn;

/// Dual-staking compute escrow vault managing active tasks and settlements.
#[derive(Debug, Default)]
pub struct ComputeEscrowVault {
    records: HashMap<TaskId, ComputeEscrowRecord>,
}

impl ComputeEscrowVault {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Locks bounty and worker bond into escrow using a structured request.
    pub fn lock_task(&mut self, params: LockTaskParams) -> Result<(), EscrowError> {
        if self.records.contains_key(&params.task_id) {
            return Err(EscrowError::AlreadyExists);
        }
        let (bounty, bond, auditor_pool, required_auditors) = match params.split {
            Some(split) => (
                split.worker_bounty,
                params.bond,
                split.auditor_pool_bounty,
                split.required_auditors,
            ),
            None => (params.bounty, params.bond, NanoSkn::ZERO, 0),
        };
        let record = ComputeEscrowRecord {
            task_id: params.task_id,
            status: EscrowStatus::Locked {
                client_pubkey: params.client_pubkey,
                worker_pubkey: params.worker_pubkey,
                bounty,
                bond,
                timeout_epoch: params.timeout_epoch,
                auditor_pool,
                auditors: Vec::new(),
                required_auditors,
            },
        };
        self.records.insert(params.task_id, record);
        Ok(())
    }

    /// Convenience wrapper for standard single-worker tasks without auditor split.
    pub fn lock_task_simple(
        &mut self,
        task_id: TaskId,
        client: [u8; 32],
        worker: [u8; 32],
        bounty: NanoSkn,
        bond: NanoSkn,
        timeout_epoch: u64,
    ) -> Result<(), EscrowError> {
        self.lock_task(LockTaskParams {
            task_id,
            client_pubkey: client,
            worker_pubkey: worker,
            bounty,
            bond,
            timeout_epoch,
            split: None,
        })
    }

    /// Convenience wrapper for tasks with delegated auditor quorum split.
    pub fn lock_task_with_split(
        &mut self,
        task_id: TaskId,
        client: [u8; 32],
        worker: [u8; 32],
        split: EscrowSplitConfig,
        worker_bond: NanoSkn,
        timeout_epoch: u64,
    ) -> Result<(), EscrowError> {
        self.lock_task(LockTaskParams {
            task_id,
            client_pubkey: client,
            worker_pubkey: worker,
            bounty: split.worker_bounty,
            bond: worker_bond,
            timeout_epoch,
            split: Some(split),
        })
    }

    /// Registers a community auditor staking bond to participate in verification.
    pub fn register_auditor(
        &mut self,
        task_id: TaskId,
        auditor_pubkey: [u8; 32],
        bond: NanoSkn,
    ) -> Result<(), EscrowError> {
        let record = self
            .records
            .get_mut(&task_id)
            .ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked {
            worker_pubkey,
            client_pubkey,
            auditors,
            ..
        } = &mut record.status
        else {
            return Err(EscrowError::NotInLockedState);
        };

        if &auditor_pubkey == worker_pubkey || &auditor_pubkey == client_pubkey {
            return Err(EscrowError::SelfAuditingForbidden);
        }
        if auditors.iter().any(|a| a.auditor_pubkey == auditor_pubkey) {
            return Err(EscrowError::AuditorAlreadyRegistered);
        }

        auditors.push(AuditorDeposit {
            auditor_pubkey,
            bond,
        });
        Ok(())
    }

    /// Settles task directly to worker upon valid submission without auditors.
    pub fn complete_task(
        &mut self,
        task_id: TaskId,
        worker: &[u8; 32],
        result_digest: [u8; 32],
        current_epoch: u64,
    ) -> Result<NanoSkn, EscrowError> {
        let record = self
            .records
            .get_mut(&task_id)
            .ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked {
            worker_pubkey,
            bounty,
            bond,
            timeout_epoch,
            ..
        } = record.status
        else {
            return Err(EscrowError::NotInLockedState);
        };

        if worker != &worker_pubkey {
            return Err(EscrowError::UnauthorizedWorker);
        }
        if current_epoch > timeout_epoch {
            return Err(EscrowError::TaskExpired {
                current: current_epoch,
                timeout: timeout_epoch,
            });
        }

        let total_payout = bounty.checked_add(bond)?;
        record.status = EscrowStatus::Completed {
            worker_pubkey,
            payout: total_payout,
            result_digest,
        };
        Ok(total_payout)
    }

    /// Settles task with quorum attestations, distributing reward to worker and auditors.
    pub fn complete_task_with_auditors(
        &mut self,
        task_id: TaskId,
        worker: &[u8; 32],
        attesting_auditors: &[[u8; 32]],
        result_digest: [u8; 32],
        current_epoch: u64,
    ) -> Result<EscrowPayoutBreakdown, EscrowError> {
        let record = self
            .records
            .get_mut(&task_id)
            .ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked {
            worker_pubkey,
            bounty,
            bond,
            timeout_epoch,
            auditor_pool,
            ref auditors,
            required_auditors,
            ..
        } = record.status
        else {
            return Err(EscrowError::NotInLockedState);
        };

        if worker != &worker_pubkey {
            return Err(EscrowError::UnauthorizedWorker);
        }
        if current_epoch > timeout_epoch {
            return Err(EscrowError::TaskExpired {
                current: current_epoch,
                timeout: timeout_epoch,
            });
        }
        if attesting_auditors.len() < required_auditors as usize {
            return Err(EscrowError::InsufficientAuditors {
                actual: attesting_auditors.len(),
                required: required_auditors as usize,
            });
        }

        let breakdown = calculate_audited_payout(
            bounty,
            bond,
            auditor_pool,
            auditors,
            attesting_auditors,
        )?;

        record.status = EscrowStatus::CompletedWithAuditors {
            worker_pubkey,
            breakdown: breakdown.clone(),
            result_digest,
        };
        Ok(breakdown)
    }

    /// Refunds client upon worker timeout and slashes worker collateral (auditors refunded in full).
    pub fn refund_task(
        &mut self,
        task_id: TaskId,
        client: &[u8; 32],
        current_epoch: u64,
    ) -> Result<(NanoSkn, NanoSkn), EscrowError> {
        let record = self
            .records
            .get_mut(&task_id)
            .ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked {
            client_pubkey,
            bounty,
            bond,
            timeout_epoch,
            auditor_pool,
            ref auditors,
            ..
        } = record.status
        else {
            return Err(EscrowError::NotInLockedState);
        };

        if client != &client_pubkey {
            return Err(EscrowError::UnauthorizedClient);
        }
        if current_epoch <= timeout_epoch {
            return Err(EscrowError::NotExpired {
                current: current_epoch,
                timeout: timeout_epoch,
            });
        }

        let compensation = NanoSkn(bond.0 / 2);
        let slashed_burn = bond.checked_sub(compensation)?;
        let total_refund = bounty.checked_add(auditor_pool)?.checked_add(compensation)?;

        let auditor_refunds = auditors
            .iter()
            .map(|a| (a.auditor_pubkey, a.bond))
            .collect();

        record.status = EscrowStatus::Refunded {
            client_pubkey,
            refund: total_refund,
            slashed_burn,
            auditor_refunds,
        };
        Ok((total_refund, slashed_burn))
    }

    pub fn get_record(&self, task_id: &TaskId) -> Option<&ComputeEscrowRecord> {
        self.records.get(task_id)
    }

    /// Verifies exact monetary conservation: locked inputs == disbursed payouts / refunds.
    pub fn total_accounted_vault_balance(&self, task_id: &TaskId) -> Result<NanoSkn, EscrowError> {
        let record = self.records.get(task_id).ok_or(EscrowError::NotFound)?;
        match &record.status {
            EscrowStatus::Locked {
                bounty,
                bond,
                auditor_pool,
                auditors,
                ..
            } => {
                let mut sum = bounty.checked_add(*bond)?.checked_add(*auditor_pool)?;
                for a in auditors {
                    sum = sum.checked_add(a.bond)?;
                }
                Ok(sum)
            }
            EscrowStatus::Completed { payout, .. } => Ok(*payout),
            EscrowStatus::CompletedWithAuditors { breakdown, .. } => Ok(breakdown.total_disbursed),
            EscrowStatus::Refunded {
                refund,
                slashed_burn,
                auditor_refunds,
                ..
            } => {
                let mut sum = refund.checked_add(*slashed_burn)?;
                for (_, b) in auditor_refunds {
                    sum = sum.checked_add(*b)?;
                }
                Ok(sum)
            }
        }
    }
}
