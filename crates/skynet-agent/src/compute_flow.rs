//! `compute_flow`: End-to-end task preparation, worker bidding, execution,
//! and cryptographic settlement pipeline for verifiable remote compute.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use skynet_cell::compute_candle::{CandleComputeExecutor, CandleTaskInput, CandleTaskOutput, ComputeCandleError};
use skynet_ledger::compute_escrow::ComputeEscrowVault;
use skynet_ledger::{EscrowError, NanoSkn};
use skynet_wire::{ComputeAssignment, ComputeBid, ComputeCompletionClaim, ComputeTaskManifest, TaskId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComputeFlowError {
    #[error("serialization error")]
    Serialization,
    #[error("neural candle compute error: {0}")]
    Candle(#[from] ComputeCandleError),
    #[error("ledger escrow error: {0}")]
    Escrow(#[from] EscrowError),
    #[error("invalid signature")]
    InvalidSignature,
    #[error("result digest mismatch")]
    DigestMismatch,
    #[error("task ID mismatch in claim or attestation")]
    TaskIdMismatch,
    #[error("worker tier {actual} lower than required {required}")]
    TierTooLow { actual: u8, required: u8 },
    #[error("worker bond is below required minimum")]
    InsufficientBond,
    #[error("duplicate auditor detected")]
    DuplicateAuditor,
}

impl From<bincode::Error> for ComputeFlowError {
    fn from(_: bincode::Error) -> Self {
        Self::Serialization
    }
}

/// Parameters for preparing a compute task manifest.
#[derive(Debug, Clone)]
pub struct PrepareTaskParams<'a> {
    pub task_id: TaskId,
    pub opcode: u16,
    pub required_tier: u8,
    pub bounty: NanoSkn,
    pub min_bond: NanoSkn,
    pub auditor_bounty: NanoSkn,
    pub required_auditors: u8,
    pub timeout_epoch: u64,
    pub input: &'a CandleTaskInput,
}

/// Parameters for assigning a compute task to a designated worker node.
#[derive(Debug, Clone)]
pub struct AssignTaskParams {
    pub task_id: TaskId,
    pub worker_pubkey: [u8; 32],
    pub bounty: NanoSkn,
    pub bond: NanoSkn,
    pub timeout_epoch: u64,
}

/// Orchestrator coordinating client-side task distribution and worker-side execution.
pub struct ComputeFlowCoordinator {
    pub escrow_vault: ComputeEscrowVault,
    pub executor: CandleComputeExecutor,
}

impl Default for ComputeFlowCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl ComputeFlowCoordinator {
    pub fn new() -> Self {
        Self {
            escrow_vault: ComputeEscrowVault::new(),
            executor: CandleComputeExecutor::new_auto(),
        }
    }

    /// Client prepares task manifest and hashes input tensor.
    pub fn prepare_task(
        client_key: &SigningKey,
        params: PrepareTaskParams,
    ) -> Result<ComputeTaskManifest, ComputeFlowError> {
        let input_bytes = bincode::serialize(params.input)?;
        let input_hash = *blake3::hash(&input_bytes).as_bytes();
        let max_ticks = (params.input.required_weights_len() + params.input.dim_in) as u64;

        Ok(ComputeTaskManifest {
            task_id: params.task_id,
            client_pubkey: client_key.verifying_key().to_bytes(),
            opcode: params.opcode,
            required_tier: params.required_tier,
            input_hash,
            timeout_epoch: params.timeout_epoch,
            max_ticks,
            bounty_nano_skn: params.bounty.0,
            min_worker_bond_nano_skn: params.min_bond.0,
            auditor_bounty_nano_skn: params.auditor_bounty.0,
            required_auditors: params.required_auditors,
        })
    }

    /// Worker evaluates manifest and constructs a signed execution bid.
    pub fn create_bid(
        worker_key: &SigningKey,
        manifest: &ComputeTaskManifest,
        worker_tier: u8,
        bond: NanoSkn,
    ) -> Result<ComputeBid, ComputeFlowError> {
        if worker_tier < manifest.required_tier {
            return Err(ComputeFlowError::TierTooLow {
                actual: worker_tier,
                required: manifest.required_tier,
            });
        }
        if bond.0 < manifest.min_worker_bond_nano_skn {
            return Err(ComputeFlowError::InsufficientBond);
        }

        Ok(ComputeBid {
            task_id: manifest.task_id,
            worker_pubkey: worker_key.verifying_key().to_bytes(),
            bond_nano_skn: bond.0,
            worker_tier,
        })
    }

    /// Client locks bounty + bond in escrow and assigns task to selected worker.
    pub fn assign_task(
        &mut self,
        client_key: &SigningKey,
        params: AssignTaskParams,
    ) -> Result<ComputeAssignment, ComputeFlowError> {
        let client_bytes = client_key.verifying_key().to_bytes();
        self.escrow_vault.lock_task_simple(
            params.task_id,
            client_bytes,
            params.worker_pubkey,
            params.bounty,
            params.bond,
            params.timeout_epoch,
        )?;

        let mut sig_payload = Vec::with_capacity(64);
        sig_payload.extend_from_slice(&params.task_id);
        sig_payload.extend_from_slice(&params.worker_pubkey);
        let sig = client_key.sign(&sig_payload);

        Ok(ComputeAssignment {
            task_id: params.task_id,
            worker_pubkey: params.worker_pubkey,
            client_signature: sig.to_bytes().to_vec(),
        })
    }

    /// Worker runs local neural compute on Candle and generates signed completion claim.
    pub fn execute_and_claim(
        &self,
        worker_key: &SigningKey,
        task_input: &CandleTaskInput,
    ) -> Result<(CandleTaskOutput, ComputeCompletionClaim), ComputeFlowError> {
        let output = self.executor.execute(task_input)?;

        let mut claim_payload = Vec::with_capacity(64);
        claim_payload.extend_from_slice(&output.task_id);
        claim_payload.extend_from_slice(&output.result_digest);
        let worker_signature = worker_key.sign(&claim_payload).to_bytes().to_vec();

        let claim = ComputeCompletionClaim {
            task_id: output.task_id,
            worker_pubkey: worker_key.verifying_key().to_bytes(),
            result_digest: output.result_digest,
            execution_ticks: output.execution_ticks,
            worker_signature,
        };
        Ok((output, claim))
    }

    /// Client verifies output digest and worker signature, releasing escrow payout.
    pub fn verify_and_settle(
        &mut self,
        task_id: TaskId,
        claim: &ComputeCompletionClaim,
        output: &CandleTaskOutput,
        current_epoch: u64,
    ) -> Result<NanoSkn, ComputeFlowError> {
        if claim.task_id != task_id || output.task_id != task_id {
            return Err(ComputeFlowError::TaskIdMismatch);
        }
        if claim.result_digest != output.result_digest {
            return Err(ComputeFlowError::DigestMismatch);
        }

        let vk = VerifyingKey::from_bytes(&claim.worker_pubkey)
            .map_err(|_| ComputeFlowError::InvalidSignature)?;
        let sig_arr: [u8; 64] = claim
            .worker_signature
            .as_slice()
            .try_into()
            .map_err(|_| ComputeFlowError::InvalidSignature)?;
        let sig = Signature::from_bytes(&sig_arr);

        let mut claim_payload = Vec::with_capacity(64);
        claim_payload.extend_from_slice(&task_id);
        claim_payload.extend_from_slice(&output.result_digest);
        vk.verify(&claim_payload, &sig)
            .map_err(|_| ComputeFlowError::InvalidSignature)?;

        let payout = self.escrow_vault.complete_task(
            task_id,
            &claim.worker_pubkey,
            output.result_digest,
            current_epoch,
        )?;
        Ok(payout)
    }

    /// Client claims refund upon worker timeout with slashing penalties applied.
    pub fn handle_timeout_refund(
        &mut self,
        task_id: TaskId,
        client_pubkey: &[u8; 32],
        current_epoch: u64,
    ) -> Result<(NanoSkn, NanoSkn), ComputeFlowError> {
        let (refund, burn) = self
            .escrow_vault
            .refund_task(task_id, client_pubkey, current_epoch)?;
        Ok((refund, burn))
    }
}

#[cfg(test)]
#[path = "compute_flow_tests.rs"]
mod tests;
