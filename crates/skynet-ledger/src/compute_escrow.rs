//! `compute_escrow`: Two-sided economic bounty escrow (`NanoSkn`), auditor quorum splitting,
//! and verifiable remote compute task locking/completion on the SkyNet ledger.

use std::collections::HashMap;

use crate::compute_escrow_settle::calculate_audited_payout;
use crate::compute_escrow_types::{
    AuditorDeposit, ComputeEscrowRecord, EscrowError, EscrowPayoutBreakdown, EscrowSplitConfig,
    EscrowStatus, LockTaskParams, TaskId,
};
use crate::escrow_slash::WorkerReputation;
use crate::streaming_escrow::ChunkEscrowSchedule;
use crate::NanoSkn;

/// Dual-staking compute escrow vault managing active tasks, PoUW reputation, and chunk schedules.
#[derive(Debug, Default)]
pub struct ComputeEscrowVault {
    pub(crate) records: HashMap<TaskId, ComputeEscrowRecord>,
    pub(crate) reputations: HashMap<[u8; 32], WorkerReputation>,
    pub(crate) chunk_schedules: HashMap<TaskId, ChunkEscrowSchedule>,
}

impl ComputeEscrowVault {
    pub fn new() -> Self {
        Self::default()
    }

    /// Locks bounty and worker bond into escrow using a structured request.
    pub fn lock_task(&mut self, params: LockTaskParams) -> Result<(), EscrowError> {
        if self.records.contains_key(&params.task_id) {
            return Err(EscrowError::AlreadyExists);
        }
        let (bounty, bond, auditor_pool, required_auditors) = match params.split {
            Some(s) => (s.worker_bounty, params.bond, s.auditor_pool_bounty, s.required_auditors),
            None => (params.bounty, params.bond, NanoSkn::ZERO, 0),
        };
        let status = EscrowStatus::Locked {
            client_pubkey: params.client_pubkey,
            worker_pubkey: params.worker_pubkey,
            bounty,
            bond,
            timeout_epoch: params.timeout_epoch,
            auditor_pool,
            auditors: Vec::new(),
            required_auditors,
        };
        self.records.insert(params.task_id, ComputeEscrowRecord { task_id: params.task_id, status });
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
    pub fn register_auditor(&mut self, task_id: TaskId, auditor_pubkey: [u8; 32], bond: NanoSkn) -> Result<(), EscrowError> {
        let record = self.records.get_mut(&task_id).ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked { worker_pubkey, client_pubkey, auditors, .. } = &mut record.status else {
            return Err(EscrowError::NotInLockedState);
        };
        if &auditor_pubkey == worker_pubkey || &auditor_pubkey == client_pubkey {
            return Err(EscrowError::SelfAuditingForbidden);
        }
        if auditors.iter().any(|a| a.auditor_pubkey == auditor_pubkey) {
            return Err(EscrowError::AuditorAlreadyRegistered);
        }
        auditors.push(AuditorDeposit { auditor_pubkey, bond });
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
        let prior = self.chunk_schedules.get(&task_id).map(|s| s.cumulative_released).unwrap_or(NanoSkn::ZERO);
        let record = self.records.get_mut(&task_id).ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked { worker_pubkey, bounty, bond, timeout_epoch, .. } = record.status else {
            return Err(EscrowError::NotInLockedState);
        };
        if worker != &worker_pubkey {
            return Err(EscrowError::UnauthorizedWorker);
        }
        if current_epoch > timeout_epoch {
            return Err(EscrowError::TaskExpired { current: current_epoch, timeout: timeout_epoch });
        }
        let total_payout = bounty.checked_add(prior)?.checked_add(bond)?;
        record.status = EscrowStatus::Completed { worker_pubkey, payout: total_payout, result_digest };
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
        let prior = self.chunk_schedules.get(&task_id).map(|s| s.cumulative_released).unwrap_or(NanoSkn::ZERO);
        let record = self.records.get_mut(&task_id).ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked { worker_pubkey, bounty, bond, timeout_epoch, auditor_pool, ref auditors, required_auditors, .. } =
            record.status
        else {
            return Err(EscrowError::NotInLockedState);
        };
        if worker != &worker_pubkey {
            return Err(EscrowError::UnauthorizedWorker);
        }
        if current_epoch > timeout_epoch {
            return Err(EscrowError::TaskExpired { current: current_epoch, timeout: timeout_epoch });
        }
        if attesting_auditors.len() < required_auditors as usize {
            return Err(EscrowError::InsufficientAuditors { actual: attesting_auditors.len(), required: required_auditors as usize });
        }
        let breakdown = calculate_audited_payout(bounty.checked_add(prior)?, bond, auditor_pool, auditors, attesting_auditors)?;
        record.status = EscrowStatus::CompletedWithAuditors { worker_pubkey, breakdown: breakdown.clone(), result_digest };
        Ok(breakdown)
    }

    pub fn get_record(&self, task_id: &TaskId) -> Option<&ComputeEscrowRecord> {
        self.records.get(task_id)
    }
}
