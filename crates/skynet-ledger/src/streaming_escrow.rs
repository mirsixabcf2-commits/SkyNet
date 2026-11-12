//! `streaming_escrow`: Per-chunk milestone streaming settlement synchronized with `SynapticVoucher`.

use serde::{Deserialize, Serialize};

use crate::compute_escrow::ComputeEscrowVault;
use crate::compute_escrow_types::{EscrowError, EscrowStatus, TaskId};
use crate::{NanoSkn, SynapticVoucher};

const CHUNK_REPUTATION_GAIN_BPS: u16 = 25;

/// Linear milestone schedule splitting a locked task's bounty across `total_chunks` slices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkEscrowSchedule {
    pub task_id: TaskId,
    pub total_chunks: u16,
    pub total_bounty: NanoSkn,
    pub released_chunks: u16,
    pub cumulative_released: NanoSkn,
}

impl ChunkEscrowSchedule {
    /// Deterministic cumulative `NanoSkn` earned after completing `0..=chunk_idx` (`100%` exact at end).
    pub fn cumulative_target_for_chunk(&self, chunk_idx: u16) -> Result<NanoSkn, EscrowError> {
        if chunk_idx >= self.total_chunks || self.total_chunks == 0 {
            return Err(EscrowError::InvalidChunkMilestone { chunk_idx, total_chunks: self.total_chunks });
        }
        let scaled = self
            .total_bounty
            .0
            .checked_mul(u128::from(chunk_idx) + 1)
            .ok_or(crate::LedgerError::Overflow)?;
        Ok(NanoSkn(scaled / u128::from(self.total_chunks)))
    }
}

/// Receipt emitted on each streaming chunk milestone release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkReleaseReceipt {
    pub task_id: TaskId,
    pub chunk_idx: u16,
    pub total_chunks: u16,
    pub incremental_released: NanoSkn,
    pub cumulative_released: NanoSkn,
    pub bond_returned: NanoSkn,
    pub expected_voucher_sequence: u64,
    pub is_final_chunk: bool,
}

/// Unilateral escrow settlement receipt when a Client disconnects mid-job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnilateralChunkSettlement {
    pub task_id: TaskId,
    pub worker_pubkey: [u8; 32],
    pub incremental_worker_payout: NanoSkn,
    pub total_worker_earned: NanoSkn,
    pub bond_returned: NanoSkn,
    pub client_unspent_refund: NanoSkn,
}

impl ComputeEscrowVault {
    /// Registers a `total_chunks` streaming schedule for a `Locked` task.
    pub fn register_chunk_schedule(&mut self, task_id: TaskId, total_chunks: u16) -> Result<ChunkEscrowSchedule, EscrowError> {
        if total_chunks == 0 {
            return Err(EscrowError::InvalidChunkSchedule);
        }
        if self.chunk_schedules.contains_key(&task_id) {
            return Err(EscrowError::ChunkScheduleAlreadyExists);
        }
        let record = self.records.get(&task_id).ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked { bounty, .. } = record.status else {
            return Err(EscrowError::NotInLockedState);
        };
        if bounty.0 < u128::from(total_chunks) {
            return Err(EscrowError::InvalidChunkSchedule);
        }
        let sched = ChunkEscrowSchedule { task_id, total_chunks, total_bounty: bounty, released_chunks: 0, cumulative_released: NanoSkn::ZERO };
        self.chunk_schedules.insert(task_id, sched.clone());
        Ok(sched)
    }

    pub fn chunk_schedule(&self, task_id: &TaskId) -> Option<&ChunkEscrowSchedule> {
        self.chunk_schedules.get(task_id)
    }

    /// Releases linear milestone payout up to `chunk_idx` (`sequence = chunk_idx as u64`).
    pub fn release_chunk_milestone(
        &mut self,
        task_id: TaskId,
        chunk_idx: u16,
        worker_pubkey: &[u8; 32],
    ) -> Result<ChunkReleaseReceipt, EscrowError> {
        let (target, prev, total_chunks, total_bounty) = {
            let s = self.chunk_schedules.get(&task_id).ok_or(EscrowError::ChunkScheduleNotFound)?;
            if chunk_idx < s.released_chunks || chunk_idx >= s.total_chunks {
                return Err(EscrowError::InvalidChunkMilestone { chunk_idx, total_chunks: s.total_chunks });
            }
            (s.cumulative_target_for_chunk(chunk_idx)?, s.cumulative_released, s.total_chunks, s.total_bounty)
        };
        let delta = target.checked_sub(prev)?;
        let is_final = chunk_idx + 1 == total_chunks;
        let bond_returned = self.apply_chunk_delta_to_record(task_id, worker_pubkey, delta, (is_final, total_bounty))?;

        let s = self.chunk_schedules.get_mut(&task_id).ok_or(EscrowError::ChunkScheduleNotFound)?;
        s.released_chunks = chunk_idx + 1;
        s.cumulative_released = target;
        self.record_verified_chunk_reputation(*worker_pubkey, CHUNK_REPUTATION_GAIN_BPS);
        Ok(ChunkReleaseReceipt {
            task_id,
            chunk_idx,
            total_chunks,
            incremental_released: delta,
            cumulative_released: target,
            bond_returned,
            expected_voucher_sequence: u64::from(chunk_idx),
            is_final_chunk: is_final,
        })
    }

    fn apply_chunk_delta_to_record(
        &mut self,
        task_id: TaskId,
        worker: &[u8; 32],
        delta: NanoSkn,
        finality: (bool, NanoSkn),
    ) -> Result<NanoSkn, EscrowError> {
        let record = self.records.get_mut(&task_id).ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked { worker_pubkey, ref mut bounty, bond, auditor_pool, ref auditors, .. } = record.status else {
            return Err(EscrowError::NotInLockedState);
        };
        if worker != &worker_pubkey {
            return Err(EscrowError::UnauthorizedWorker);
        }
        *bounty = bounty.checked_sub(delta)?;
        if finality.0 && auditor_pool == NanoSkn::ZERO && auditors.is_empty() {
            let payout = finality.1.checked_add(bond)?;
            record.status = EscrowStatus::Completed { worker_pubkey, payout, result_digest: [0u8; 32] };
            return Ok(bond);
        }
        Ok(NanoSkn::ZERO)
    }

    /// Unilaterally closes a streaming task when Client disconnects, backed by a signed `SynapticVoucher`.
    pub fn settle_unilateral_with_voucher(
        &mut self,
        task_id: TaskId,
        v: &SynapticVoucher,
        max_allowance: NanoSkn,
    ) -> Result<UnilateralChunkSettlement, EscrowError> {
        v.verify(max_allowance)?;
        let s = self.chunk_schedules.get(&task_id).ok_or(EscrowError::ChunkScheduleNotFound)?;
        if v.cumulative_amount > s.total_bounty {
            return Err(EscrowError::VoucherExceedsTaskBounty { voucher_amount: v.cumulative_amount.0, total_bounty: s.total_bounty.0 });
        }
        let (total_bounty, prev_released) = (s.total_bounty, s.cumulative_released);

        let record = self.records.get_mut(&task_id).ok_or(EscrowError::NotFound)?;
        let EscrowStatus::Locked { client_pubkey, worker_pubkey, bond, auditor_pool, ref auditors, .. } = record.status else {
            return Err(EscrowError::NotInLockedState);
        };
        if v.payee_pubkey != worker_pubkey {
            return Err(EscrowError::UnauthorizedWorker);
        }
        let incremental = v.cumulative_amount.checked_sub(prev_released)?;
        let client_refund = total_bounty.checked_sub(v.cumulative_amount)?.checked_add(auditor_pool)?;
        let auditor_refunds = auditors.iter().map(|a| (a.auditor_pubkey, a.bond)).collect();
        record.status = EscrowStatus::Refunded { client_pubkey, refund: client_refund, slashed_burn: v.cumulative_amount.checked_add(bond)?, auditor_refunds };
        if let Some(sched) = self.chunk_schedules.get_mut(&task_id) {
            sched.cumulative_released = v.cumulative_amount;
        }
        Ok(UnilateralChunkSettlement { task_id, worker_pubkey, incremental_worker_payout: incremental, total_worker_earned: v.cumulative_amount, bond_returned: bond, client_unspent_refund: client_refund })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HumanWallet, TxOutput, UtxoDagLedger, UtxoId, VoucherChannelTracker};

    #[test]
    fn ten_chunks_streaming_conserves_non_divisible_nanoskn() {
        let mut vault = ComputeEscrowVault::new();
        let (task_a, client, worker) = ([1u8; 32], [2u8; 32], [3u8; 32]);
        vault.lock_task_simple(task_a, client, worker, NanoSkn(1_003), NanoSkn(200), 50).unwrap();
        vault.register_chunk_schedule(task_a, 10).unwrap();
        let initial_a = vault.total_accounted_vault_balance(&task_a).unwrap();

        let mut sum = NanoSkn::ZERO;
        for idx in 0..10u16 {
            let r = vault.release_chunk_milestone(task_a, idx, &worker).unwrap();
            sum = sum.checked_add(r.incremental_released).unwrap();
            assert_eq!(vault.total_accounted_vault_balance(&task_a).unwrap(), initial_a);
        }
        assert_eq!(sum, NanoSkn(1_003));
    }

    #[test]
    fn unilateral_disconnect_at_chunk_six_settles_voucher_and_escrow() {
        let mut vault = ComputeEscrowVault::new();
        let human = HumanWallet::from_master_seed([9u8; 32]);
        let (payer_key, _) = human.authorize_agent(0, NanoSkn(5_000), 100);
        let worker_key = human.derive_agent_hotkey(1);
        let worker_pk = worker_key.verifying_key().to_bytes();
        let bond_out = TxOutput::agent_bond(human.human_pubkey(), payer_key.verifying_key().to_bytes(), NanoSkn(10_000), NanoSkn(5_000));
        let (mut ledger, ghash) = UtxoDagLedger::from_genesis(vec![bond_out.clone()]).unwrap();
        let bond_id = UtxoId { tx_hash: ghash, vout: 0 };

        let task_b = [7u8; 32];
        vault.lock_task_simple(task_b, payer_key.verifying_key().to_bytes(), worker_pk, NanoSkn(1_000), NanoSkn(250), 80).unwrap();
        let sched = vault.register_chunk_schedule(task_b, 10).unwrap();
        let initial_b = vault.total_accounted_vault_balance(&task_b).unwrap();

        let v5 = SynapticVoucher::new_signed(&payer_key, bond_id, worker_pk, 5, sched.cumulative_target_for_chunk(5).unwrap());
        let mut tracker = VoucherChannelTracker::new(bond_id, NanoSkn(5_000));
        tracker.receive_udp_voucher(v5.clone()).unwrap();

        let uni = vault.settle_unilateral_with_voucher(task_b, &v5, NanoSkn(5_000)).unwrap();
        assert_eq!((uni.total_worker_earned, uni.bond_returned, uni.client_unspent_refund), (NanoSkn(600), NanoSkn(250), NanoSkn(400)));
        assert_eq!(vault.total_accounted_vault_balance(&task_b).unwrap(), initial_b);

        let (tx, v_proof) = tracker.build_unilateral_settlement_tx(&worker_key, &bond_out, NanoSkn::ONE_NANO, vec![ghash], 1).unwrap();
        let (thash, _) = ledger.admit_unilateral_voucher_settlement(tx, &v_proof).unwrap();
        ledger.finalize_tx(&thash).unwrap();
        assert_eq!(ledger.balance_of(&worker_pk), NanoSkn(600));
    }
}
