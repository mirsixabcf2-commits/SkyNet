//! `compute_escrow_types`: Type definitions, data structures, and errors for compute bounty escrows.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{LedgerError, NanoSkn};

pub type TaskId = [u8; 32];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EscrowError {
    #[error("task escrow already exists for task id")]
    AlreadyExists,
    #[error("task escrow not found for task id")]
    NotFound,
    #[error("escrow record is not in locked state")]
    NotInLockedState,
    #[error("unauthorized worker attempted to claim completion")]
    UnauthorizedWorker,
    #[error("unauthorized client attempted to request refund")]
    UnauthorizedClient,
    #[error("insufficient auditor attestations: got {actual}, required {required}")]
    InsufficientAuditors { actual: usize, required: usize },
    #[error("unregistered auditor in attestation list")]
    UnregisteredAuditor,
    #[error("duplicate auditor detected in attestation list")]
    DuplicateAuditor,
    #[error("worker cannot audit own task")]
    SelfAuditingForbidden,
    #[error("auditor already registered for this task")]
    AuditorAlreadyRegistered,
    #[error("task has not expired yet (current: {current}, timeout: {timeout})")]
    NotExpired { current: u64, timeout: u64 },
    #[error("task has expired and cannot be completed (current: {current}, timeout: {timeout})")]
    TaskExpired { current: u64, timeout: u64 },
    #[error("ledger arithmetic error: {0}")]
    Ledger(#[from] LedgerError),
}

/// Deposit and bond provided by a community auditor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditorDeposit {
    pub auditor_pubkey: [u8; 32],
    pub bond: NanoSkn,
}

/// Configuration splitting total bounty between executor worker and auditor quorum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscrowSplitConfig {
    pub worker_bounty: NanoSkn,
    pub auditor_pool_bounty: NanoSkn,
    pub required_auditors: u8,
}

/// Parameters for locking a task escrow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockTaskParams {
    pub task_id: TaskId,
    pub client_pubkey: [u8; 32],
    pub worker_pubkey: [u8; 32],
    pub bounty: NanoSkn,
    pub bond: NanoSkn,
    pub timeout_epoch: u64,
    pub split: Option<EscrowSplitConfig>,
}

/// Itemized payout statement disbursed upon successful audited settlement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscrowPayoutBreakdown {
    pub worker_payout: NanoSkn,
    pub auditor_payouts: Vec<([u8; 32], NanoSkn)>,
    pub total_disbursed: NanoSkn,
}

/// Lifecycle status of a compute task bounty escrow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowStatus {
    Locked {
        client_pubkey: [u8; 32],
        worker_pubkey: [u8; 32],
        bounty: NanoSkn,
        bond: NanoSkn,
        timeout_epoch: u64,
        auditor_pool: NanoSkn,
        auditors: Vec<AuditorDeposit>,
        required_auditors: u8,
    },
    Completed {
        worker_pubkey: [u8; 32],
        payout: NanoSkn,
        result_digest: [u8; 32],
    },
    CompletedWithAuditors {
        worker_pubkey: [u8; 32],
        breakdown: EscrowPayoutBreakdown,
        result_digest: [u8; 32],
    },
    Refunded {
        client_pubkey: [u8; 32],
        refund: NanoSkn,
        slashed_burn: NanoSkn,
        auditor_refunds: Vec<([u8; 32], NanoSkn)>,
    },
}

/// Dedicated record tracking a compute escrow instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComputeEscrowRecord {
    pub task_id: TaskId,
    pub status: EscrowStatus,
}
