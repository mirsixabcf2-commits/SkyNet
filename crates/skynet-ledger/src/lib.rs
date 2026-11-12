//! `skynet-ledger`: 21-Trillion SKN fixed-cap monetary engine (`NanoSkn` over `u128`),
//! deterministic Mainnet `GENESIS_HASH` (`85%` Vault + `15%` Circulating), Soul-Body Custody
//! (`HumanPrincipalDiversionBlocked`, `Sovereign Reclaim`, `0%`-burn `Yield Sweep`),
//! `~1 KB` Stateless `Utreexo` Merkle Forest, PoUW fraud-slashing & streaming compute escrows,
//! loss-tolerant `SynapticVoucher` channels, and UTXO-DAG ledger.

use thiserror::Error;

pub mod compute_escrow;
pub mod compute_escrow_refund;
pub mod compute_escrow_settle;
pub mod compute_escrow_types;
pub mod custody;
pub mod dag_ledger;
pub mod escrow_slash;
pub mod genesis;
pub mod halving_vault;
pub mod hwid_airdrop;
pub mod nano_skn;
pub mod streaming_escrow;
pub mod utreexo;
pub mod voucher;
pub mod yield_sweep;

#[cfg(test)]
mod ledger_core_tests;
#[cfg(test)]
mod ledger_custody_tests;

pub use compute_escrow::ComputeEscrowVault;
pub use compute_escrow_types::{
    AuditorDeposit, ComputeEscrowRecord, EscrowError, EscrowPayoutBreakdown, EscrowSplitConfig,
    EscrowStatus, LockTaskParams, TaskId, TaskId as ComputeTaskId,
};
pub use custody::{
    AgentFundingParams, AgentOwnershipCertificate, CustodyPolicy, HumanWallet, SknTransaction,
    TxOutput, UtxoId,
};
pub use dag_ledger::UtxoDagLedger;
pub use escrow_slash::{
    FraudProofClaim, SlashOutcome, WorkerReputation, DEFAULT_POUW_SCORE_BPS, MAX_POUW_SCORE_BPS,
};
pub use genesis::{
    compute_genesis_hash, default_mainnet_genesis, genesis_for_network,
    validate_genesis_invariants, GenesisAllocation, GenesisConfig, GenesisUtxoEntry,
    DEFAULT_MAINNET_NETWORK_NAME, GENESIS_CIRCULATING_NANO_SKN, GENESIS_CIRCULATING_SKN,
    GENESIS_HASH_DOMAIN, GENESIS_VAULT_NANO_SKN, GENESIS_VAULT_SKN,
};
pub use halving_vault::GenesisHalvingVault;
pub use hwid_airdrop::{
    HwidClaimRecord, HwidClaimRequest, HwidDigest, MarketingBootstrapVault, MarketingError,
    ProofDigest,
};
pub use nano_skn::{
    AgentMetabolicBank, NanoSkn, DEFAULT_EPOCHS_PER_YEAR, GENESIS_MAX_SKN, GENESIS_TOTAL_NANO_SKN,
    MIN_TX_NANO_SKN, NANO_PER_SKN,
};
pub use streaming_escrow::{
    ChunkEscrowSchedule, ChunkReleaseReceipt, UnilateralChunkSettlement,
};
pub use utreexo::{StatelessUtreexoRoots, UtreexoForestBuilder, UtreexoProof};
pub use voucher::{SynapticVoucher, VoucherChannelTracker};
pub use yield_sweep::{
    build_yield_sweep_tx, plan_yield_sweep, YieldSweepConfig, YieldSweepOrder,
    DEFAULT_AUTO_SWEEP_NANO_SKN, DEFAULT_AUTO_SWEEP_SKN,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LedgerError {
    #[error("nanoSKN arithmetic overflow")]
    Overflow,
    #[error("insufficient nanoSKN balance (available {available}, required {required})")]
    InsufficientBalance { available: u128, required: u128 },
    #[error("genesis allocation exceeds 21 trillion SKN cap")]
    ExceedsGenesisCap,
    #[error("genesis network_name or halving schedule metadata is invalid")]
    InvalidGenesisMetadata,
    #[error("genesis supply invariant violated: vault ({vault}) + circulating ({circulating}) != {expected_total}")]
    GenesisSupplyMismatch {
        vault: u128,
        circulating: u128,
        expected_total: u128,
    },
    #[error("transaction must have at least one input and one output")]
    EmptyInputsOrOutputs,
    #[error("duplicate UTXO input inside the same transaction")]
    DuplicateInput,
    #[error("output amount must be at least 1 nanoSKN")]
    ZeroOutputAmount,
    #[error("referenced UTXO not found or already spent")]
    UnknownOrSpentUtxo,
    #[error("signer is not authorized by UTXO custody policy")]
    UnauthorizedSpend,
    #[error("agent operator attempted to divert principal away from human owner")]
    HumanPrincipalDiversionBlocked,
    #[error("synaptic voucher exceeds human-approved allowance ({requested} > {allowance})")]
    VoucherExceedsAllowance { requested: u128, allowance: u128 },
    #[error("synaptic voucher is stale or non-monotonic")]
    StaleVoucher,
    #[error("invalid Ed25519 signature")]
    InvalidSignature,
    #[error("invalid Utreexo stateless Merkle inclusion proof")]
    InvalidUtreexoProof,
    #[error("UTXO conservation violated: inputs ({inputs}) != outputs ({outputs}) + fee ({fee})")]
    ConservationMismatch {
        inputs: u128,
        outputs: u128,
        fee: u128,
    },
    #[error("transaction not found in pending DAG")]
    UnknownTransaction,
    #[error("cellular bloodstream invariant mismatch: on_chain ({on_chain}) != blood ({blood}) + cells ({cells})")]
    CellularInvariantMismatch {
        on_chain: u128,
        blood: u128,
        cells: u128,
    },
}
