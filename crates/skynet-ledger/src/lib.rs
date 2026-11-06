//! `skynet-ledger`: 21-Trillion SKN fixed-cap monetary engine (`NanoSkn` over `u128`),
//! annual 50% halving schedule, Human Sovereign Wallet & Agent Bond custody,
//! `~1 KB` Stateless `Utreexo` Merkle Forest accumulator, loss-tolerant `SynapticVoucher`
//! micro-payment channels, intra-agent cellular ATP accounting, and UTXO-DAG ledger.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// `1 SKN = 1,000,000,000 nanoSKN` (`10^9`).
pub const NANO_PER_SKN: u128 = 1_000_000_000;

/// Hard-capped total supply minted once at Genesis: `21,000,000,000,000 SKN` (21 Trillion SKN).
pub const GENESIS_MAX_SKN: u128 = 21_000_000_000_000;

/// Exact atomic Genesis supply in `nanoSKN`: `2.1 * 10^22 nanoSKN` (requires `u128`).
pub const GENESIS_TOTAL_NANO_SKN: u128 = GENESIS_MAX_SKN * NANO_PER_SKN;

/// Minimum transferable / metabolic unit: `1 nanoSKN`.
pub const MIN_TX_NANO_SKN: u128 = 1;

/// Default number of 1-minute epochs per calendar year (`365 * 24 * 60 = 525,600`).
pub const DEFAULT_EPOCHS_PER_YEAR: u64 = 525_600;

pub mod compute_escrow_types;
pub mod compute_escrow_settle;
pub mod compute_escrow;
pub use compute_escrow::ComputeEscrowVault;
pub use compute_escrow_types::{
    AuditorDeposit, ComputeEscrowRecord, EscrowError, EscrowPayoutBreakdown, EscrowSplitConfig,
    EscrowStatus, LockTaskParams, TaskId as ComputeTaskId,
};

pub mod hwid_airdrop;
pub use hwid_airdrop::{
    HwidClaimRecord, HwidClaimRequest, HwidDigest, MarketingBootstrapVault, MarketingError,
    ProofDigest,
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LedgerError {
    #[error("nanoSKN arithmetic overflow")]
    Overflow,
    #[error("insufficient nanoSKN balance (available {available}, required {required})")]
    InsufficientBalance { available: u128, required: u128 },
    #[error("genesis allocation exceeds 21 trillion SKN cap")]
    ExceedsGenesisCap,
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

/// Type-safe atomic SKN quantity (`1 nanoSKN = 10^-9 SKN`) backed by `u128`.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct NanoSkn(pub u128);

impl NanoSkn {
    pub const ZERO: Self = Self(0);
    pub const ONE_NANO: Self = Self(1);
    pub const GENESIS_CAP: Self = Self(GENESIS_TOTAL_NANO_SKN);

    pub const fn from_nano(nano: u128) -> Self {
        Self(nano)
    }

    pub fn from_skn(skn: u128) -> Result<Self, LedgerError> {
        skn.checked_mul(NANO_PER_SKN)
            .map(Self)
            .ok_or(LedgerError::Overflow)
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self, LedgerError> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(LedgerError::Overflow)
    }

    pub fn checked_sub(self, rhs: Self) -> Result<Self, LedgerError> {
        self.0
            .checked_sub(rhs.0)
            .map(Self)
            .ok_or(LedgerError::InsufficientBalance {
                available: self.0,
                required: rhs.0,
            })
    }

    pub const fn whole_skn(self) -> u128 {
        self.0 / NANO_PER_SKN
    }

    pub const fn sub_nano(self) -> u128 {
        self.0 % NANO_PER_SKN
    }
}

impl fmt::Display for NanoSkn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:09} SKN", self.whole_skn(), self.sub_nano())
    }
}

/// Enforces the fixed `21,000,000,000,000 SKN` Genesis supply and 50% annual halving schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisHalvingVault {
    pub epochs_per_year: u64,
    pub initial_seed_circulating: NanoSkn,
    pub initial_reserve_locked: NanoSkn,
    pub remaining_reserve: NanoSkn,
    pub distributed_from_reserve: NanoSkn,
    pub next_epoch_to_release: u64,
}

impl GenesisHalvingVault {
    pub fn new(seed_circulating: NanoSkn, epochs_per_year: u64) -> Result<Self, LedgerError> {
        if seed_circulating.0 > GENESIS_TOTAL_NANO_SKN {
            return Err(LedgerError::ExceedsGenesisCap);
        }
        let epochs = epochs_per_year.max(1);
        let reserve = NanoSkn(GENESIS_TOTAL_NANO_SKN - seed_circulating.0);
        Ok(Self {
            epochs_per_year: epochs,
            initial_seed_circulating: seed_circulating,
            initial_reserve_locked: reserve,
            remaining_reserve: reserve,
            distributed_from_reserve: NanoSkn::ZERO,
            next_epoch_to_release: 0,
        })
    }

    pub fn annual_quota_for_year(&self, year_index: u64) -> NanoSkn {
        let shift = year_index.saturating_add(1);
        if shift >= 128 {
            NanoSkn::ZERO
        } else {
            NanoSkn(self.initial_reserve_locked.0 >> shift)
        }
    }

    pub fn scheduled_epoch_emission(&self, epoch: u64) -> NanoSkn {
        if self.remaining_reserve.0 == 0 {
            return NanoSkn::ZERO;
        }
        let year_index = epoch / self.epochs_per_year;
        let annual = self.annual_quota_for_year(year_index).0;
        let per_epoch = annual / (self.epochs_per_year as u128);
        NanoSkn(per_epoch.min(self.remaining_reserve.0))
    }

    pub fn release_next_epoch(&mut self) -> Result<NanoSkn, LedgerError> {
        let emission = self.scheduled_epoch_emission(self.next_epoch_to_release);
        self.remaining_reserve = self.remaining_reserve.checked_sub(emission)?;
        self.distributed_from_reserve = self.distributed_from_reserve.checked_add(emission)?;
        self.next_epoch_to_release = self.next_epoch_to_release.saturating_add(1);
        debug_assert_eq!(self.total_accounted_supply()?.0, GENESIS_TOTAL_NANO_SKN);
        Ok(emission)
    }

    pub fn total_accounted_supply(&self) -> Result<NanoSkn, LedgerError> {
        self.initial_seed_circulating
            .checked_add(self.distributed_from_reserve)?
            .checked_add(self.remaining_reserve)
    }
}

/// Unique identifier of an unspent SKN output in the DAG.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct UtxoId {
    pub tx_hash: [u8; 32],
    pub vout: u16,
}

/// Custody covenant separating **Human Sovereign Cold Ownership** from **AI Agent Hot Operation**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyPolicy {
    /// Standard personal/cold wallet UTXO: 0% metabolic decay, spendable ONLY by `owner_pubkey`.
    DirectOwner,
    /// Delegated AI Organism Bond:
    /// - `human_owner` holds ultimate sovereign recovery rights (can sweep/withdraw 100% anytime,
    ///   even if the AI Agent hardware dies or gets compromised).
    /// - `owner_pubkey` on `TxOutput` is the online `AgentNode` hot key, which may use the UTXO
    ///   for cellular ATP & consensus, and spend at most `max_voucher_allowance` externally while
    ///   returning all remaining principal back to `human_owner`!
    AgentBond {
        human_owner: [u8; 32],
        max_voucher_allowance: NanoSkn,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOutput {
    /// Active spend/consensus key (Human pubkey for `DirectOwner`, or Agent pubkey for `AgentBond`).
    pub owner_pubkey: [u8; 32],
    pub amount: NanoSkn,
    pub policy: CustodyPolicy,
}

impl TxOutput {
    pub fn direct(owner_pubkey: [u8; 32], amount: NanoSkn) -> Self {
        Self {
            owner_pubkey,
            amount,
            policy: CustodyPolicy::DirectOwner,
        }
    }

    pub fn agent_bond(
        human_owner: [u8; 32],
        agent_pubkey: [u8; 32],
        amount: NanoSkn,
        max_voucher_allowance: NanoSkn,
    ) -> Self {
        Self {
            owner_pubkey: agent_pubkey,
            amount,
            policy: CustodyPolicy::AgentBond {
                human_owner,
                max_voucher_allowance,
            },
        }
    }

    pub fn human_beneficiary(&self) -> [u8; 32] {
        match &self.policy {
            CustodyPolicy::DirectOwner => self.owner_pubkey,
            CustodyPolicy::AgentBond { human_owner, .. } => *human_owner,
        }
    }
}

/// Signed offline certificate issued by a Human Wallet authorizing an `AgentNode` hot key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentOwnershipCertificate {
    pub human_owner_pubkey: [u8; 32],
    pub agent_pubkey: [u8; 32],
    pub agent_index: u32,
    pub max_voucher_allowance: NanoSkn,
    pub valid_until_epoch: u64,
    pub human_signature: Vec<u8>,
}

impl AgentOwnershipCertificate {
    pub fn signing_digest(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"SKYNET_HUMAN_AGENT_CERT_V1");
        h.update(&self.human_owner_pubkey);
        h.update(&self.agent_pubkey);
        h.update(&self.agent_index.to_le_bytes());
        h.update(&self.max_voucher_allowance.0.to_le_bytes());
        h.update(&self.valid_until_epoch.to_le_bytes());
        *h.finalize().as_bytes()
    }

    pub fn verify(&self, current_epoch: u64) -> Result<(), LedgerError> {
        if current_epoch > self.valid_until_epoch {
            return Err(LedgerError::UnauthorizedSpend);
        }
        let vk = VerifyingKey::from_bytes(&self.human_owner_pubkey)
            .map_err(|_| LedgerError::InvalidSignature)?;
        let sig_arr: [u8; 64] = self
            .human_signature
            .as_slice()
            .try_into()
            .map_err(|_| LedgerError::InvalidSignature)?;
        vk.verify(&self.signing_digest(), &Signature::from_bytes(&sig_arr))
            .map_err(|_| LedgerError::InvalidSignature)
    }
}

/// Parameters for funding an AI Agent from a Human's cold UTXO.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentFundingParams {
    pub cold_utxo_id: UtxoId,
    pub cold_utxo_amount: NanoSkn,
    pub agent_pubkey: [u8; 32],
    pub bond_amount: NanoSkn,
    pub max_voucher_allowance: NanoSkn,
    pub fee: NanoSkn,
    pub dag_parents: Vec<[u8; 32]>,
    pub nonce: u64,
}

/// Deterministic Hierarchical Human Wallet (`BLAKE3` KDF):
/// One human master seed manages cold personal SKN vaults AND any number of AI Agent Nodes,
/// never exposing the human's master private key on online Agent hardware.
pub struct HumanWallet {
    master_seed: [u8; 32],
    master_signing_key: SigningKey,
}

impl HumanWallet {
    /// Creates or restores a Human Wallet from a mnemonic/passphrase or raw 32-byte master seed.
    pub fn from_recovery_phrase(phrase: &str) -> Self {
        let seed = blake3::derive_key("skynet/wallet/master-seed/v1", phrase.as_bytes());
        Self::from_master_seed(seed)
    }

    pub fn from_master_seed(master_seed: [u8; 32]) -> Self {
        let key_bytes = blake3::derive_key("skynet/wallet/human-cold-ed25519/v1", &master_seed);
        let master_signing_key = SigningKey::from_bytes(&key_bytes);
        Self {
            master_seed,
            master_signing_key,
        }
    }

    pub fn human_pubkey(&self) -> [u8; 32] {
        self.master_signing_key.verifying_key().to_bytes()
    }

    pub fn master_signing_key(&self) -> &SigningKey {
        &self.master_signing_key
    }

    /// Deterministically derives an isolated operational hot key for AI Agent `#agent_index`.
    pub fn derive_agent_hotkey(&self, agent_index: u32) -> SigningKey {
        let mut ctx = Vec::with_capacity(36);
        ctx.extend_from_slice(&self.master_seed);
        ctx.extend_from_slice(&agent_index.to_le_bytes());
        let derived = blake3::derive_key("skynet/wallet/agent-hotkey-ed25519/v1", &ctx);
        SigningKey::from_bytes(&derived)
    }

    /// Issues an offline-signed delegation certificate for an AI Agent Node.
    pub fn authorize_agent(
        &self,
        agent_index: u32,
        max_voucher_allowance: NanoSkn,
        valid_until_epoch: u64,
    ) -> (SigningKey, AgentOwnershipCertificate) {
        let agent_key = self.derive_agent_hotkey(agent_index);
        let mut cert = AgentOwnershipCertificate {
            human_owner_pubkey: self.human_pubkey(),
            agent_pubkey: agent_key.verifying_key().to_bytes(),
            agent_index,
            max_voucher_allowance,
            valid_until_epoch,
            human_signature: Vec::new(),
        };
        let sig = self.master_signing_key.sign(&cert.signing_digest());
        cert.human_signature = sig.to_bytes().to_vec();
        (agent_key, cert)
    }

    /// Funds a new or existing AI Agent from the Human's cold UTXO while keeping sovereign ownership.
    pub fn create_agent_funding_tx(
        &self,
        params: AgentFundingParams,
    ) -> Result<SknTransaction, LedgerError> {
        let total_out = params.bond_amount.checked_add(params.fee)?;
        let change = params.cold_utxo_amount.checked_sub(total_out)?;
        let mut outputs = vec![TxOutput::agent_bond(
            self.human_pubkey(),
            params.agent_pubkey,
            params.bond_amount,
            params.max_voucher_allowance,
        )];
        if change.0 > 0 {
            outputs.push(TxOutput::direct(self.human_pubkey(), change));
        }
        Ok(SknTransaction::new_signed(
            &self.master_signing_key,
            vec![params.cold_utxo_id],
            outputs,
            params.fee,
            params.dag_parents,
            params.nonce,
        ))
    }

    /// Sovereign Human Recovery: Reclaims 100% of an AI Agent's bonded SKN back to the Human's
    /// cold wallet (works even if the Agent hardware is offline, dead, or compromised!).
    pub fn build_sovereign_reclaim_tx(
        &self,
        bond_utxo_id: UtxoId,
        bond_amount: NanoSkn,
        fee: NanoSkn,
        dag_parents: Vec<[u8; 32]>,
        nonce: u64,
    ) -> Result<SknTransaction, LedgerError> {
        let reclaimed = bond_amount.checked_sub(fee)?;
        Ok(SknTransaction::new_signed(
            &self.master_signing_key,
            vec![bond_utxo_id],
            vec![TxOutput::direct(self.human_pubkey(), reclaimed)],
            fee,
            dag_parents,
            nonce,
        ))
    }
}

/// Loss-tolerant cumulative off-chain micro-payment voucher over UDP (`<= 200 bytes`).
/// If intermediate UDP packets drop, any newer voucher (`sequence`, `cumulative_amount`)
/// seamlessly supersedes all lost packets with zero retransmissions!
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SynapticVoucher {
    pub bond_utxo: UtxoId,
    pub payer_agent_pubkey: [u8; 32],
    pub payee_pubkey: [u8; 32],
    pub sequence: u64,
    pub cumulative_amount: NanoSkn,
    pub signature: Vec<u8>,
}

impl SynapticVoucher {
    pub fn new_signed(
        agent_signing_key: &SigningKey,
        bond_utxo: UtxoId,
        payee_pubkey: [u8; 32],
        sequence: u64,
        cumulative_amount: NanoSkn,
    ) -> Self {
        let payer_agent_pubkey = agent_signing_key.verifying_key().to_bytes();
        let mut v = Self {
            bond_utxo,
            payer_agent_pubkey,
            payee_pubkey,
            sequence,
            cumulative_amount,
            signature: Vec::new(),
        };
        let sig = agent_signing_key.sign(&v.signing_digest());
        v.signature = sig.to_bytes().to_vec();
        v
    }

    pub fn signing_digest(&self) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"SKYNET_SYNAPTIC_VOUCHER_V1");
        h.update(&self.bond_utxo.tx_hash);
        h.update(&self.bond_utxo.vout.to_le_bytes());
        h.update(&self.payer_agent_pubkey);
        h.update(&self.payee_pubkey);
        h.update(&self.sequence.to_le_bytes());
        h.update(&self.cumulative_amount.0.to_le_bytes());
        *h.finalize().as_bytes()
    }

    pub fn verify(&self, max_allowance: NanoSkn) -> Result<(), LedgerError> {
        if self.cumulative_amount.0 < MIN_TX_NANO_SKN {
            return Err(LedgerError::ZeroOutputAmount);
        }
        if self.cumulative_amount > max_allowance {
            return Err(LedgerError::VoucherExceedsAllowance {
                requested: self.cumulative_amount.0,
                allowance: max_allowance.0,
            });
        }
        let vk = VerifyingKey::from_bytes(&self.payer_agent_pubkey)
            .map_err(|_| LedgerError::InvalidSignature)?;
        let sig_arr: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| LedgerError::InvalidSignature)?;
        vk.verify(&self.signing_digest(), &Signature::from_bytes(&sig_arr))
            .map_err(|_| LedgerError::InvalidSignature)
    }
}

/// Payee-side stateless/lightweight tracker absorbing lossy UDP `SynapticVoucher` streams
/// and settling thousands of micro-inferences in a single on-chain transaction.
#[derive(Debug, Clone)]
pub struct VoucherChannelTracker {
    pub bond_utxo: UtxoId,
    pub max_allowance: NanoSkn,
    pub highest_voucher: Option<SynapticVoucher>,
}

impl VoucherChannelTracker {
    pub fn new(bond_utxo: UtxoId, max_allowance: NanoSkn) -> Self {
        Self {
            bond_utxo,
            max_allowance,
            highest_voucher: None,
        }
    }

    /// Absorbs an incoming UDP voucher, returning the incremental `NanoSkn` earned even if
    /// arbitrary prior UDP vouchers dropped in transit!
    pub fn receive_udp_voucher(&mut self, voucher: SynapticVoucher) -> Result<NanoSkn, LedgerError> {
        if voucher.bond_utxo != self.bond_utxo {
            return Err(LedgerError::UnknownOrSpentUtxo);
        }
        voucher.verify(self.max_allowance)?;

        if let Some(prev) = &self.highest_voucher {
            if voucher.payer_agent_pubkey != prev.payer_agent_pubkey
                || voucher.payee_pubkey != prev.payee_pubkey
            {
                return Err(LedgerError::UnauthorizedSpend);
            }
            if voucher.sequence <= prev.sequence
                || voucher.cumulative_amount <= prev.cumulative_amount
            {
                return Err(LedgerError::StaleVoucher);
            }
            let delta = voucher.cumulative_amount.checked_sub(prev.cumulative_amount)?;
            self.highest_voucher = Some(voucher);
            Ok(delta)
        } else {
            let earned = voucher.cumulative_amount;
            self.highest_voucher = Some(voucher);
            Ok(earned)
        }
    }

    /// Builds the single settlement transaction closing the voucher session while returning
    /// 100% of unspent bonded principal safely locked to `human_owner`!
    pub fn build_cooperative_settlement_tx(
        &self,
        payer_agent_key: &SigningKey,
        bond_output: &TxOutput,
        fee: NanoSkn,
        dag_parents: Vec<[u8; 32]>,
        nonce: u64,
    ) -> Result<SknTransaction, LedgerError> {
        let voucher = self
            .highest_voucher
            .as_ref()
            .ok_or(LedgerError::EmptyInputsOrOutputs)?;
        let CustodyPolicy::AgentBond {
            human_owner,
            max_voucher_allowance,
        } = bond_output.policy
        else {
            return Err(LedgerError::UnauthorizedSpend);
        };
        voucher.verify(max_voucher_allowance)?;

        let deducted = voucher.cumulative_amount.checked_add(fee)?;
        let refund_to_human_bond = bond_output.amount.checked_sub(deducted)?;
        let new_remaining_allowance = max_voucher_allowance.checked_sub(voucher.cumulative_amount)?;

        let mut outputs = vec![TxOutput::direct(
            voucher.payee_pubkey,
            voucher.cumulative_amount,
        )];
        if refund_to_human_bond.0 > 0 {
            outputs.push(TxOutput::agent_bond(
                human_owner,
                bond_output.owner_pubkey,
                refund_to_human_bond,
                new_remaining_allowance,
            ));
        }

        Ok(SknTransaction::new_signed(
            payer_agent_key,
            vec![self.bond_utxo],
            outputs,
            fee,
            dag_parents,
            nonce,
        ))
    }

    /// Unilateral Payee Settlement: The Payee builds and signs a settlement transaction,
    /// proving their authorization via the Payer's cryptographically signed `SynapticVoucher`.
    /// This prevents malicious or offline Payers from defaulting or holding earned funds hostage!
    pub fn build_unilateral_settlement_tx(
        &self,
        payee_key: &SigningKey,
        bond_output: &TxOutput,
        fee: NanoSkn,
        dag_parents: Vec<[u8; 32]>,
        nonce: u64,
    ) -> Result<(SknTransaction, SynapticVoucher), LedgerError> {
        let voucher = self
            .highest_voucher
            .as_ref()
            .ok_or(LedgerError::EmptyInputsOrOutputs)?;
        let CustodyPolicy::AgentBond {
            human_owner,
            max_voucher_allowance,
        } = bond_output.policy
        else {
            return Err(LedgerError::UnauthorizedSpend);
        };
        voucher.verify(max_voucher_allowance)?;

        let deducted = voucher.cumulative_amount.checked_add(fee)?;
        let refund_to_human_bond = bond_output.amount.checked_sub(deducted)?;
        let new_remaining_allowance = max_voucher_allowance.checked_sub(voucher.cumulative_amount)?;

        let mut outputs = vec![TxOutput::direct(
            voucher.payee_pubkey,
            voucher.cumulative_amount,
        )];
        if refund_to_human_bond.0 > 0 {
            outputs.push(TxOutput::agent_bond(
                human_owner,
                bond_output.owner_pubkey,
                refund_to_human_bond,
                new_remaining_allowance,
            ));
        }

        let tx = SknTransaction::new_signed(
            payee_key,
            vec![self.bond_utxo],
            outputs,
            fee,
            dag_parents,
            nonce,
        );
        Ok((tx, voucher.clone()))
    }
}

/// Compact Merkle Inclusion Proof (`~64 - 384 bytes`) allowing `~1 KB` RAM peripheral nodes
/// to verify UTXOs statelessly inside a single `<= 1200B` UDP packet!
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtreexoProof {
    pub leaf_position: u64,
    pub tree_height: u8,
    pub siblings: Vec<[u8; 32]>,
}

/// Ultra-lightweight **Utreexo Binary Merkle Forest Accumulator** (`<= 2 KB` RAM total!):
/// Stores at most 64 roots (`roots[h]` holds the root of a perfect binary tree of height `h`
/// whenever bit `h` of `num_leaves` is `1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatelessUtreexoRoots {
    pub num_leaves: u64,
    /// Only active roots `(height, root_hash)` — typically `<= 32` entries (`~1 KB` RAM!).
    pub forest_roots: Vec<(u8, [u8; 32])>,
}

impl Default for StatelessUtreexoRoots {
    fn default() -> Self {
        Self::new()
    }
}

impl StatelessUtreexoRoots {
    pub fn new() -> Self {
        Self {
            num_leaves: 0,
            forest_roots: Vec::new(),
        }
    }

    /// Exact RAM footprint of the active cryptographic roots in bytes (`~33 bytes * popcount`).
    pub fn ram_footprint_bytes(&self) -> usize {
        std::mem::size_of::<u64>() + self.forest_roots.len() * 33
    }

    pub fn hash_utxo_leaf(id: &UtxoId, out: &TxOutput) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"SKYNET_UTREEXO_LEAF_V1");
        h.update(&id.tx_hash);
        h.update(&id.vout.to_le_bytes());
        h.update(&out.owner_pubkey);
        h.update(&out.amount.0.to_le_bytes());
        match &out.policy {
            CustodyPolicy::DirectOwner => {
                h.update(&[0u8]);
            }
            CustodyPolicy::AgentBond {
                human_owner,
                max_voucher_allowance,
            } => {
                h.update(&[1u8]);
                h.update(human_owner);
                h.update(&max_voucher_allowance.0.to_le_bytes());
            }
        }
        *h.finalize().as_bytes()
    }

    pub fn hash_parent(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"SKYNET_UTREEXO_NODE_V1");
        h.update(left);
        h.update(right);
        *h.finalize().as_bytes()
    }

    /// Appends a new UTXO leaf in $O(\log N)$ time using `< 2 KB` of state.
    pub fn add_leaf(&mut self, mut carry: [u8; 32]) {
        let mut h = 0u8;
        while let Some(&(_, prev_root)) = self.forest_roots.iter().find(|&&(ht, _)| ht == h) {
            self.forest_roots.retain(|&(ht, _)| ht != h);
            carry = Self::hash_parent(&prev_root, &carry);
            h += 1;
        }
        self.forest_roots.push((h, carry));
        self.forest_roots.sort_by_key(|&(ht, _)| ht);
        self.num_leaves += 1;
    }

    /// Statelessly verifies a UTXO's existence in `< 10 microseconds` using ONLY `~1 KB` of roots!
    pub fn verify_utxo(
        &self,
        id: &UtxoId,
        out: &TxOutput,
        proof: &UtreexoProof,
    ) -> Result<(), LedgerError> {
        if proof.siblings.len() != proof.tree_height as usize {
            return Err(LedgerError::InvalidUtreexoProof);
        }
        let expected_root = self
            .forest_roots
            .iter()
            .find(|&&(ht, _)| ht == proof.tree_height)
            .map(|&(_, r)| r)
            .ok_or(LedgerError::InvalidUtreexoProof)?;

        let mut curr = Self::hash_utxo_leaf(id, out);
        let mut pos = proof.leaf_position;
        for sib in &proof.siblings {
            if pos & 1 == 0 {
                curr = Self::hash_parent(&curr, sib);
            } else {
                curr = Self::hash_parent(sib, &curr);
            }
            pos >>= 1;
        }

        if curr == expected_root {
            Ok(())
        } else {
            Err(LedgerError::InvalidUtreexoProof)
        }
    }
}

/// Builds Utreexo proofs from an active UTXO snapshot (used by wallets / cluster provers).
pub struct UtreexoForestBuilder {
    leaves: Vec<[u8; 32]>,
    index_of: HashMap<UtxoId, usize>,
    pub roots: StatelessUtreexoRoots,
}

impl UtreexoForestBuilder {
    pub fn from_utxos<'a>(utxos: impl IntoIterator<Item = (&'a UtxoId, &'a TxOutput)>) -> Self {
        let mut leaves = Vec::new();
        let mut index_of = HashMap::new();
        let mut roots = StatelessUtreexoRoots::new();

        for (i, (id, out)) in utxos.into_iter().enumerate() {
            let leaf = StatelessUtreexoRoots::hash_utxo_leaf(id, out);
            leaves.push(leaf);
            index_of.insert(*id, i);
            roots.add_leaf(leaf);
        }

        Self {
            leaves,
            index_of,
            roots,
        }
    }

    pub fn prove(&self, id: &UtxoId) -> Option<UtreexoProof> {
        let global_idx = *self.index_of.get(id)?;
        let n = self.leaves.len();

        // Partition `0..n` into power-of-two trees from highest bit to lowest bit
        let mut offset = 0usize;
        for shift in (0..64).rev() {
            let size = 1usize << shift;
            if (n & size) != 0 {
                if global_idx >= offset && global_idx < offset + size {
                    let local_idx = global_idx - offset;
                    let mut level: Vec<[u8; 32]> = self.leaves[offset..offset + size].to_vec();
                    let mut siblings = Vec::with_capacity(shift);
                    let mut cur = local_idx;
                    while level.len() > 1 {
                        let sib_idx = cur ^ 1;
                        siblings.push(level[sib_idx]);
                        let mut next_level = Vec::with_capacity(level.len() / 2);
                        #[allow(clippy::chunks_exact_to_as_chunks)]
                        for pair in level.chunks_exact(2) {
                            next_level.push(StatelessUtreexoRoots::hash_parent(&pair[0], &pair[1]));
                        }
                        level = next_level;
                        cur >>= 1;
                    }
                    return Some(UtreexoProof {
                        leaf_position: local_idx as u64,
                        tree_height: shift as u8,
                        siblings,
                    });
                }
                offset += size;
            }
        }
        None
    }
}

/// Signed SKN UTXO-DAG transaction (compact enough to fit inside a single `<= 1200B` UDP datagram).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SknTransaction {
    pub inputs: Vec<UtxoId>,
    pub outputs: Vec<TxOutput>,
    pub fee: NanoSkn,
    pub dag_parents: Vec<[u8; 32]>,
    pub nonce: u64,
    pub sender_pubkey: [u8; 32],
    pub signature_bytes: Vec<u8>,
}

impl SknTransaction {
    pub fn new_signed(
        signing_key: &SigningKey,
        inputs: Vec<UtxoId>,
        outputs: Vec<TxOutput>,
        fee: NanoSkn,
        dag_parents: Vec<[u8; 32]>,
        nonce: u64,
    ) -> Self {
        let sender_pubkey = signing_key.verifying_key().to_bytes();
        let mut tx = Self {
            inputs,
            outputs,
            fee,
            dag_parents,
            nonce,
            sender_pubkey,
            signature_bytes: Vec::new(),
        };
        let digest = tx.signing_digest();
        let sig: Signature = signing_key.sign(&digest);
        tx.signature_bytes = sig.to_bytes().to_vec();
        tx
    }

    pub fn signing_digest(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SKYNET_TX_V2");
        hasher.update(&(self.inputs.len() as u32).to_le_bytes());
        for inp in &self.inputs {
            hasher.update(&inp.tx_hash);
            hasher.update(&inp.vout.to_le_bytes());
        }
        hasher.update(&(self.outputs.len() as u32).to_le_bytes());
        for out in &self.outputs {
            hasher.update(&out.owner_pubkey);
            hasher.update(&out.amount.0.to_le_bytes());
            match &out.policy {
                CustodyPolicy::DirectOwner => {
                    hasher.update(&[0u8]);
                }
                CustodyPolicy::AgentBond {
                    human_owner,
                    max_voucher_allowance,
                } => {
                    hasher.update(&[1u8]);
                    hasher.update(human_owner);
                    hasher.update(&max_voucher_allowance.0.to_le_bytes());
                }
            }
        }
        hasher.update(&self.fee.0.to_le_bytes());
        hasher.update(&(self.dag_parents.len() as u32).to_le_bytes());
        for parent in &self.dag_parents {
            hasher.update(parent);
        }
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.sender_pubkey);
        *hasher.finalize().as_bytes()
    }

    pub fn tx_hash(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.signing_digest());
        hasher.update(&self.signature_bytes);
        *hasher.finalize().as_bytes()
    }

    pub fn verify_signature(&self) -> Result<(), LedgerError> {
        let vk = VerifyingKey::from_bytes(&self.sender_pubkey)
            .map_err(|_| LedgerError::InvalidSignature)?;
        let sig_arr: [u8; 64] = self
            .signature_bytes
            .as_slice()
            .try_into()
            .map_err(|_| LedgerError::InvalidSignature)?;
        let sig = Signature::from_bytes(&sig_arr);
        vk.verify(&self.signing_digest(), &sig)
            .map_err(|_| LedgerError::InvalidSignature)
    }
}

/// Cluster-level UTXO-DAG ledger tracking live UTXOs, conflict sets, and the Cluster Nutrient Pool.
#[derive(Debug, Clone)]
pub struct UtxoDagLedger {
    utxos: BTreeMap<UtxoId, TxOutput>,
    pending_txs: HashMap<[u8; 32], SknTransaction>,
    conflict_sets: HashMap<UtxoId, BTreeSet<[u8; 32]>>,
    finalized_tips: BTreeSet<[u8; 32]>,
    pub cluster_nutrient_pool: NanoSkn,
}

impl UtxoDagLedger {
    pub fn from_genesis(genesis_outputs: Vec<TxOutput>) -> Result<(Self, [u8; 32]), LedgerError> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SKYNET_GENESIS_VERTEX");
        let mut total = NanoSkn::ZERO;
        for out in &genesis_outputs {
            if out.amount.0 < MIN_TX_NANO_SKN {
                return Err(LedgerError::ZeroOutputAmount);
            }
            total = total.checked_add(out.amount)?;
            hasher.update(&out.owner_pubkey);
            hasher.update(&out.amount.0.to_le_bytes());
        }
        if total.0 > GENESIS_TOTAL_NANO_SKN {
            return Err(LedgerError::ExceedsGenesisCap);
        }
        let genesis_hash = *hasher.finalize().as_bytes();

        let mut utxos = BTreeMap::new();
        for (idx, out) in genesis_outputs.into_iter().enumerate() {
            utxos.insert(
                UtxoId {
                    tx_hash: genesis_hash,
                    vout: idx as u16,
                },
                out,
            );
        }

        let mut finalized_tips = BTreeSet::new();
        finalized_tips.insert(genesis_hash);

        Ok((
            Self {
                utxos,
                pending_txs: HashMap::new(),
                conflict_sets: HashMap::new(),
                finalized_tips,
                cluster_nutrient_pool: NanoSkn::ZERO,
            },
            genesis_hash,
        ))
    }

    /// Exports a compact `~1 KB` Stateless `Utreexo` root commitment + proof builder for light nodes.
    pub fn build_utreexo_forest(&self) -> UtreexoForestBuilder {
        UtreexoForestBuilder::from_utxos(self.utxos.iter())
    }

    pub fn get_utxo(&self, id: &UtxoId) -> Option<&TxOutput> {
        self.utxos.get(id)
    }

    pub fn utxos_for_owner(&self, owner_pubkey: &[u8; 32]) -> Vec<(UtxoId, TxOutput)> {
        self.utxos
            .iter()
            .filter(|(_, out)| &out.owner_pubkey == owner_pubkey)
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    pub fn balance_of(&self, owner_pubkey: &[u8; 32]) -> NanoSkn {
        let sum: u128 = self
            .utxos
            .values()
            .filter(|out| &out.owner_pubkey == owner_pubkey)
            .map(|out| out.amount.0)
            .sum();
        NanoSkn(sum)
    }

    /// Total net worth belonging to a Human Owner (both Direct cold UTXOs + all bonded AI Agents!).
    pub fn human_total_net_worth(&self, human_pubkey: &[u8; 32]) -> NanoSkn {
        let sum: u128 = self
            .utxos
            .values()
            .filter(|out| &out.human_beneficiary() == human_pubkey)
            .map(|out| out.amount.0)
            .sum();
        NanoSkn(sum)
    }

    pub fn finalized_tips(&self) -> Vec<[u8; 32]> {
        self.finalized_tips.iter().copied().collect()
    }

    pub fn all_utxos(&self) -> Vec<(UtxoId, TxOutput)> {
        self.utxos.iter().map(|(k, v)| (*k, v.clone())).collect()
    }

    pub fn from_raw_state(
        utxos: Vec<(UtxoId, TxOutput)>,
        finalized_tips: Vec<[u8; 32]>,
        cluster_nutrient_pool: NanoSkn,
    ) -> Self {
        Self {
            utxos: utxos.into_iter().collect(),
            pending_txs: HashMap::new(),
            conflict_sets: HashMap::new(),
            finalized_tips: finalized_tips.into_iter().collect(),
            cluster_nutrient_pool,
        }
    }

    pub fn conflict_candidates(&self, utxo: &UtxoId) -> Option<&BTreeSet<[u8; 32]>> {
        self.conflict_sets.get(utxo)
    }

    pub fn pending_tx(&self, tx_hash: &[u8; 32]) -> Option<&SknTransaction> {
        self.pending_txs.get(tx_hash)
    }

    /// Validates transaction signatures, `CustodyPolicy` covenants (protecting Human Principal!),
    /// and exact `NanoSkn` conservation.
    pub fn admit_pending_tx(
        &mut self,
        tx: SknTransaction,
    ) -> Result<([u8; 32], Vec<[u8; 32]>), LedgerError> {
        if tx.inputs.is_empty() || tx.outputs.is_empty() {
            return Err(LedgerError::EmptyInputsOrOutputs);
        }
        tx.verify_signature()?;

        let mut seen_inputs = BTreeSet::new();
        let mut input_sum = NanoSkn::ZERO;
        // Track `(human_owner, min_protected_principal)` whenever an Agent hot key spends a Bond!
        let mut protected_human_principal: BTreeMap<[u8; 32], NanoSkn> = BTreeMap::new();

        for inp in &tx.inputs {
            if !seen_inputs.insert(*inp) {
                return Err(LedgerError::DuplicateInput);
            }
            let utxo = self
                .utxos
                .get(inp)
                .ok_or(LedgerError::UnknownOrSpentUtxo)?;

            match &utxo.policy {
                CustodyPolicy::DirectOwner => {
                    if utxo.owner_pubkey != tx.sender_pubkey {
                        return Err(LedgerError::UnauthorizedSpend);
                    }
                }
                CustodyPolicy::AgentBond {
                    human_owner,
                    max_voucher_allowance,
                } => {
                    if tx.sender_pubkey == *human_owner {
                        // Sovereign Human Owner override: full unrestricted spend/reclaim allowed!
                    } else if tx.sender_pubkey == utxo.owner_pubkey {
                        // AI Agent hot-key spend: external spend + fee MUST NOT exceed allowance!
                        let max_spend_plus_fee = max_voucher_allowance.checked_add(tx.fee)?;
                        let min_return = if utxo.amount > max_spend_plus_fee {
                            utxo.amount.checked_sub(max_spend_plus_fee)?
                        } else {
                            NanoSkn::ZERO
                        };
                        let entry = protected_human_principal
                            .entry(*human_owner)
                            .or_insert(NanoSkn::ZERO);
                        *entry = entry.checked_add(min_return)?;
                    } else {
                        return Err(LedgerError::UnauthorizedSpend);
                    }
                }
            }
            input_sum = input_sum.checked_add(utxo.amount)?;
        }

        let mut output_sum = NanoSkn::ZERO;
        let mut returned_to_human: BTreeMap<[u8; 32], NanoSkn> = BTreeMap::new();
        for out in &tx.outputs {
            if out.amount.0 < MIN_TX_NANO_SKN {
                return Err(LedgerError::ZeroOutputAmount);
            }
            output_sum = output_sum.checked_add(out.amount)?;
            let beneficiary = out.human_beneficiary();
            let entry = returned_to_human
                .entry(beneficiary)
                .or_insert(NanoSkn::ZERO);
            *entry = entry.checked_add(out.amount)?;
        }

        // Enforce Human Principal Covenant: an AI Agent can NEVER steal its Human Owner's principal!
        for (human, min_required) in protected_human_principal {
            let actual_returned = returned_to_human
                .get(&human)
                .copied()
                .unwrap_or(NanoSkn::ZERO);
            if actual_returned < min_required {
                return Err(LedgerError::HumanPrincipalDiversionBlocked);
            }
        }

        let required = output_sum.checked_add(tx.fee)?;
        if input_sum != required {
            return Err(LedgerError::ConservationMismatch {
                inputs: input_sum.0,
                outputs: output_sum.0,
                fee: tx.fee.0,
            });
        }

        let tx_hash = tx.tx_hash();
        let mut conflicts = BTreeSet::new();
        for inp in &tx.inputs {
            let entry = self.conflict_sets.entry(*inp).or_default();
            for existing in entry.iter() {
                if *existing != tx_hash {
                    conflicts.insert(*existing);
                }
            }
            entry.insert(tx_hash);
        }

        self.pending_txs.insert(tx_hash, tx);
        Ok((tx_hash, conflicts.into_iter().collect()))
    }

    /// Admits a unilateral payee settlement transaction verified by a valid `SynapticVoucher`.
    pub fn admit_unilateral_voucher_settlement(
        &mut self,
        tx: SknTransaction,
        voucher: &SynapticVoucher,
    ) -> Result<([u8; 32], Vec<[u8; 32]>), LedgerError> {
        if tx.inputs.len() != 1 || tx.outputs.is_empty() {
            return Err(LedgerError::EmptyInputsOrOutputs);
        }
        tx.verify_signature()?;

        let bond_utxo_id = tx.inputs[0];
        if bond_utxo_id != voucher.bond_utxo {
            return Err(LedgerError::UnknownOrSpentUtxo);
        }
        let bond_out = self.utxos.get(&bond_utxo_id).ok_or(LedgerError::UnknownOrSpentUtxo)?;

        let CustodyPolicy::AgentBond {
            human_owner,
            max_voucher_allowance,
        } = bond_out.policy
        else {
            return Err(LedgerError::UnauthorizedSpend);
        };

        if bond_out.owner_pubkey != voucher.payer_agent_pubkey {
            return Err(LedgerError::UnauthorizedSpend);
        }
        if tx.sender_pubkey != voucher.payee_pubkey {
            return Err(LedgerError::UnauthorizedSpend);
        }
        voucher.verify(max_voucher_allowance)?;

        if tx.outputs[0].owner_pubkey != voucher.payee_pubkey
            || tx.outputs[0].amount != voucher.cumulative_amount
        {
            return Err(LedgerError::UnauthorizedSpend);
        }

        let deducted = voucher.cumulative_amount.checked_add(tx.fee)?;
        let expected_refund = bond_out.amount.checked_sub(deducted)?;
        if expected_refund.0 > 0
            && (tx.outputs.len() < 2
                || tx.outputs[1].human_beneficiary() != human_owner
                || tx.outputs[1].amount != expected_refund)
        {
            return Err(LedgerError::HumanPrincipalDiversionBlocked);
        }

        let total_out = tx
            .outputs
            .iter()
            .try_fold(NanoSkn::ZERO, |acc, o| acc.checked_add(o.amount))?;
        if bond_out.amount != total_out.checked_add(tx.fee)? {
            return Err(LedgerError::ConservationMismatch {
                inputs: bond_out.amount.0,
                outputs: total_out.0,
                fee: tx.fee.0,
            });
        }

        let tx_hash = tx.tx_hash();
        let mut conflicts = BTreeSet::new();
        let entry = self.conflict_sets.entry(bond_utxo_id).or_default();
        for existing in entry.iter() {
            if *existing != tx_hash {
                conflicts.insert(*existing);
            }
        }
        entry.insert(tx_hash);
        self.pending_txs.insert(tx_hash, tx);
        Ok((tx_hash, conflicts.into_iter().collect()))
    }

    pub fn finalize_tx(&mut self, tx_hash: &[u8; 32]) -> Result<Vec<UtxoId>, LedgerError> {
        let tx = self
            .pending_txs
            .remove(tx_hash)
            .ok_or(LedgerError::UnknownTransaction)?;

        for inp in &tx.inputs {
            if !self.utxos.contains_key(inp) {
                return Err(LedgerError::UnknownOrSpentUtxo);
            }
        }

        let mut losing_txs = BTreeSet::new();
        for inp in &tx.inputs {
            self.utxos.remove(inp);
            if let Some(contenders) = self.conflict_sets.remove(inp) {
                for c in contenders {
                    if &c != tx_hash {
                        losing_txs.insert(c);
                    }
                }
            }
        }
        for loser in &losing_txs {
            if let Some(loser_tx) = self.pending_txs.remove(loser) {
                for other_inp in &loser_tx.inputs {
                    if let Some(set) = self.conflict_sets.get_mut(other_inp) {
                        set.remove(loser);
                    }
                }
            }
        }

        let mut minted = Vec::with_capacity(tx.outputs.len());
        for (idx, out) in tx.outputs.into_iter().enumerate() {
            let id = UtxoId {
                tx_hash: *tx_hash,
                vout: idx as u16,
            };
            self.utxos.insert(id, out);
            minted.push(id);
        }

        for parent in &tx.dag_parents {
            self.finalized_tips.remove(parent);
        }
        self.finalized_tips.insert(*tx_hash);
        self.cluster_nutrient_pool = self.cluster_nutrient_pool.checked_add(tx.fee)?;

        Ok(minted)
    }

    pub fn total_tracked_supply(&self) -> Result<NanoSkn, LedgerError> {
        let mut sum = self.cluster_nutrient_pool;
        for out in self.utxos.values() {
            sum = sum.checked_add(out.amount)?;
        }
        Ok(sum)
    }
}

/// Enforces the Two-Tier Isomorphic Conservation Law inside a single AI `AgentNode`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetabolicBank {
    pub blood_pool: NanoSkn,
}

impl AgentMetabolicBank {
    pub fn new(initial_blood: NanoSkn) -> Self {
        Self {
            blood_pool: initial_blood,
        }
    }

    pub fn deposit_to_blood(&mut self, amount: NanoSkn) -> Result<(), LedgerError> {
        self.blood_pool = self.blood_pool.checked_add(amount)?;
        Ok(())
    }

    pub fn draw_from_blood(&mut self, amount: NanoSkn) -> Result<NanoSkn, LedgerError> {
        self.blood_pool = self.blood_pool.checked_sub(amount)?;
        Ok(amount)
    }

    pub fn verify_invariant(
        &self,
        expected_total: NanoSkn,
        cell_energies: impl IntoIterator<Item = NanoSkn>,
    ) -> Result<(), LedgerError> {
        let mut cells_sum = NanoSkn::ZERO;
        for e in cell_energies {
            cells_sum = cells_sum.checked_add(e)?;
        }
        let actual = self.blood_pool.checked_add(cells_sum)?;
        if actual != expected_total {
            return Err(LedgerError::CellularInvariantMismatch {
                on_chain: expected_total.0,
                blood: self.blood_pool.0,
                cells: cells_sum.0,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn genesis_cap_and_annual_halving_preserve_exact_21_trillion_skn() {
        let seed = NanoSkn::from_skn(1_000_000_000_000).unwrap();
        let mut vault = GenesisHalvingVault::new(seed, 100).unwrap();

        assert_eq!(
            vault.annual_quota_for_year(0),
            NanoSkn::from_skn(10_000_000_000_000).unwrap()
        );
        assert_eq!(
            vault.annual_quota_for_year(1),
            NanoSkn::from_skn(5_000_000_000_000).unwrap()
        );
        assert_eq!(
            vault.annual_quota_for_year(2),
            NanoSkn::from_skn(2_500_000_000_000).unwrap()
        );

        for _ in 0..300 {
            vault.release_next_epoch().unwrap();
            assert_eq!(
                vault.total_accounted_supply().unwrap().0,
                GENESIS_TOTAL_NANO_SKN
            );
        }
    }

    #[test]
    fn utxo_dag_detects_double_spend_and_conserves_nanoskn() {
        let mut rng = ChaCha8Rng::seed_from_u64(11);
        let alice = SigningKey::generate(&mut rng);
        let bob = SigningKey::generate(&mut rng);
        let carol = SigningKey::generate(&mut rng);

        let initial = NanoSkn::from_nano(5_000_000_001);
        let (mut ledger, gen_hash) = UtxoDagLedger::from_genesis(vec![TxOutput::direct(
            alice.verifying_key().to_bytes(),
            initial,
        )])
        .unwrap();

        let utxo0 = UtxoId {
            tx_hash: gen_hash,
            vout: 0,
        };

        let tx_to_bob = SknTransaction::new_signed(
            &alice,
            vec![utxo0],
            vec![TxOutput::direct(
                bob.verifying_key().to_bytes(),
                NanoSkn::from_nano(5_000_000_000),
            )],
            NanoSkn::ONE_NANO,
            vec![gen_hash],
            1,
        );
        let tx_to_carol = SknTransaction::new_signed(
            &alice,
            vec![utxo0],
            vec![TxOutput::direct(
                carol.verifying_key().to_bytes(),
                NanoSkn::from_nano(4_999_999_990),
            )],
            NanoSkn::from_nano(11),
            vec![gen_hash],
            2,
        );

        let (h1, c1) = ledger.admit_pending_tx(tx_to_bob).unwrap();
        assert!(c1.is_empty());
        let (h2, c2) = ledger.admit_pending_tx(tx_to_carol).unwrap();
        assert_eq!(c2, vec![h1]);

        ledger.finalize_tx(&h1).unwrap();
        assert_eq!(
            ledger.balance_of(&bob.verifying_key().to_bytes()),
            NanoSkn::from_nano(5_000_000_000)
        );
        assert_eq!(ledger.total_tracked_supply().unwrap(), initial);
        assert!(ledger.finalize_tx(&h2).is_err());
    }

    #[test]
    fn human_wallet_protects_principal_against_compromised_agent_and_supports_emergency_reclaim() {
        let human = HumanWallet::from_recovery_phrase("pain skynet sovereign human seed 2026");
        let thief = HumanWallet::from_recovery_phrase("evil attacker wallet");

        let initial_skn = NanoSkn::from_skn(1_000).unwrap();
        let bond_skn = NanoSkn::from_skn(400).unwrap();
        let allowance = NanoSkn::from_skn(10).unwrap(); // Agent can spend at most 10 SKN!

        let (mut ledger, gen_hash) =
            UtxoDagLedger::from_genesis(vec![TxOutput::direct(human.human_pubkey(), initial_skn)])
                .unwrap();

        let (agent_key, cert) = human.authorize_agent(0, allowance, 100);
        assert!(cert.verify(5).is_ok());

        // 1. Human funds AI Agent #0 while retaining sovereign ownership
        let fund_tx = human
            .create_agent_funding_tx(AgentFundingParams {
                cold_utxo_id: UtxoId {
                    tx_hash: gen_hash,
                    vout: 0,
                },
                cold_utxo_amount: initial_skn,
                agent_pubkey: cert.agent_pubkey,
                bond_amount: bond_skn,
                max_voucher_allowance: allowance,
                fee: NanoSkn::ONE_NANO,
                dag_parents: vec![gen_hash],
                nonce: 1,
            })
            .unwrap();
        let (fund_hash, _) = ledger.admit_pending_tx(fund_tx).unwrap();
        let minted = ledger.finalize_tx(&fund_hash).unwrap();
        let bond_utxo_id = minted[0];

        // Human's total net worth is still 1000 SKN - 1 nanoSKN fee!
        assert_eq!(
            ledger.human_total_net_worth(&human.human_pubkey()),
            initial_skn.checked_sub(NanoSkn::ONE_NANO).unwrap()
        );

        // 2. Suppose hackers steal `agent_key` and try to drain 350 SKN (> 10 SKN allowance!) to `thief`
        let hack_tx = SknTransaction::new_signed(
            &agent_key,
            vec![bond_utxo_id],
            vec![
                TxOutput::direct(
                    thief.human_pubkey(),
                    NanoSkn::from_skn(350).unwrap(),
                ),
                TxOutput::direct(
                    human.human_pubkey(),
                    NanoSkn::from_skn(50)
                        .unwrap()
                        .checked_sub(NanoSkn::ONE_NANO)
                        .unwrap(),
                ),
            ],
            NanoSkn::ONE_NANO,
            vec![fund_hash],
            2,
        );
        assert_eq!(
            ledger.admit_pending_tx(hack_tx),
            Err(LedgerError::HumanPrincipalDiversionBlocked)
        );

        // 3. Human owner executes emergency sovereign reclaim using ONLY their cold master key!
        let reclaim_tx = human
            .build_sovereign_reclaim_tx(
                bond_utxo_id,
                bond_skn,
                NanoSkn::ONE_NANO,
                vec![fund_hash],
                3,
            )
            .unwrap();
        let (rec_hash, _) = ledger.admit_pending_tx(reclaim_tx).unwrap();
        ledger.finalize_tx(&rec_hash).unwrap();

        assert_eq!(ledger.balance_of(&cert.agent_pubkey), NanoSkn::ZERO);
        assert_eq!(
            ledger.balance_of(&human.human_pubkey()),
            initial_skn.checked_sub(NanoSkn::from_nano(2)).unwrap()
        );
    }

    #[test]
    fn utreexo_verifies_utxos_in_under_500_bytes_of_ram_and_vouchers_survive_udp_loss() {
        let human = HumanWallet::from_recovery_phrase("human-owner-1");
        let payee_agent = HumanWallet::from_recovery_phrase("provider-agent").derive_agent_hotkey(0);
        let (payer_agent, _) =
            human.authorize_agent(0, NanoSkn::from_nano(10_000), 50);

        // Build 500 UTXOs and verify StatelessUtreexoRoots takes < 500 bytes of RAM!
        let mut genesis_outs = Vec::new();
        for i in 0..500u128 {
            genesis_outs.push(TxOutput::agent_bond(
                human.human_pubkey(),
                payer_agent.verifying_key().to_bytes(),
                NanoSkn::from_nano(100_000 + i),
                NanoSkn::from_nano(10_000),
            ));
        }
        let (mut ledger, gen_hash) = UtxoDagLedger::from_genesis(genesis_outs).unwrap();
        let forest = ledger.build_utreexo_forest();
        assert!(forest.roots.ram_footprint_bytes() < 500);

        let target_id = UtxoId {
            tx_hash: gen_hash,
            vout: 0,
        };
        let target_out = ledger.get_utxo(&target_id).unwrap().clone();
        let proof = forest.prove(&target_id).unwrap();
        forest
            .roots
            .verify_utxo(&target_id, &target_out, &proof)
            .unwrap();

        // Stream 100 SynapticVouchers (5 nanoSKN each) dropping 70% of UDP packets randomly!
        let mut tracker = VoucherChannelTracker::new(target_id, NanoSkn::from_nano(10_000));
        let mut total_credited = NanoSkn::ZERO;
        for seq in 1..=100u64 {
            let v = SynapticVoucher::new_signed(
                &payer_agent,
                target_id,
                payee_agent.verifying_key().to_bytes(),
                seq,
                NanoSkn::from_nano(seq as u128 * 5),
            );
            // Drop every packet except multiples of 25 (`#25`, `#50`, `#75`, `#100`)!
            if seq % 25 != 0 {
                continue;
            }
            let delta = tracker.receive_udp_voucher(v).unwrap();
            total_credited = total_credited.checked_add(delta).unwrap();
        }
        // Despite 96% packet drop, payee still received exact 500 nanoSKN and settles in 1 tx!
        assert_eq!(total_credited, NanoSkn::from_nano(500));

        let settle_tx = tracker
            .build_cooperative_settlement_tx(
                &payer_agent,
                &target_out,
                NanoSkn::ONE_NANO,
                vec![gen_hash],
                1,
            )
            .unwrap();
        let (shash, _) = ledger.admit_pending_tx(settle_tx).unwrap();
        ledger.finalize_tx(&shash).unwrap();
        assert_eq!(
            ledger.balance_of(&payee_agent.verifying_key().to_bytes()),
            NanoSkn::from_nano(500)
        );
    }

    #[test]
    fn test_unilateral_voucher_settlement_without_payer_cooperation() {
        let mut rng = ChaCha8Rng::seed_from_u64(8888);
        let human = HumanWallet::from_master_seed([0x77u8; 32]);
        let payer_agent = SigningKey::generate(&mut rng);
        let payee_agent = SigningKey::generate(&mut rng);

        let initial_bond = NanoSkn::from_nano(50_000);
        let allowance = NanoSkn::from_nano(5_000);
        let bond_out = TxOutput::agent_bond(
            human.human_pubkey(),
            payer_agent.verifying_key().to_bytes(),
            initial_bond,
            allowance,
        );

        let (mut ledger, gen_hash) = UtxoDagLedger::from_genesis(vec![bond_out.clone()]).unwrap();
        let bond_utxo = UtxoId {
            tx_hash: gen_hash,
            vout: 0,
        };

        // Payer streams vouchers to payee
        let mut tracker = VoucherChannelTracker::new(bond_utxo, allowance);
        let voucher = SynapticVoucher::new_signed(
            &payer_agent,
            bond_utxo,
            payee_agent.verifying_key().to_bytes(),
            1,
            NanoSkn::from_nano(2_500),
        );
        tracker.receive_udp_voucher(voucher).unwrap();

        // Payer goes offline / disappears! Payee settles unilaterally on-chain!
        let fee = NanoSkn::ONE_NANO;
        let (unilateral_tx, highest_v) = tracker
            .build_unilateral_settlement_tx(&payee_agent, &bond_out, fee, vec![gen_hash], 1)
            .unwrap();

        let (tx_hash, _) = ledger
            .admit_unilateral_voucher_settlement(unilateral_tx, &highest_v)
            .expect("unilateral settlement must be admitted with valid voucher");
        let minted = ledger.finalize_tx(&tx_hash).unwrap();
        assert_eq!(minted.len(), 2);

        // Payee received exact earned 2500 nanoSKN
        assert_eq!(
            ledger.balance_of(&payee_agent.verifying_key().to_bytes()),
            NanoSkn::from_nano(2_500)
        );
        // Human owner still owns remaining bonded principal
        assert_eq!(
            ledger.human_total_net_worth(&human.human_pubkey()),
            initial_bond.checked_sub(NanoSkn::from_nano(2_500)).unwrap().checked_sub(fee).unwrap()
        );
    }
}
