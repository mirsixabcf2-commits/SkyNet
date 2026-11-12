//! `opportunistic_worker`: End-to-end edge worker engine orchestrating `FountainDecoder`
//! shard reassembly, pause/resume `execute_micro_train_slices`, `< 10ms` fast inference,
//! `Canary` spot-check fraud reporting, streaming `ChunkVoucherDelivery`, and `Yield Sweep`.

use std::collections::{BTreeSet, HashMap};

use ed25519_dalek::SigningKey;
use skynet_cell::compute_candle::{CandleTaskInput, CandleTaskOutput};
use skynet_cell::{
    execute_micro_train_slices, verify_canary_spot_check, CanaryConfig, CanaryVerdict, CheckpointError,
    MicroTrainBatch, MicroTrainCheckpoint, MicroTrainConfig, MicroTrainError, MicroTrainOutcome,
    MicroTrainResult, PreemptGuard, DEFAULT_MAX_GRAD_NORM_MILLI,
};
use skynet_ledger::compute_escrow::ComputeEscrowVault;
use skynet_ledger::{
    build_yield_sweep_tx, CustodyPolicy, EscrowError, FraudProofClaim, LedgerError, NanoSkn,
    SknTransaction, SlashOutcome, TxOutput, UtxoDagLedger, UtxoId, VoucherChannelTracker, YieldSweepConfig,
};
use skynet_wire::{
    try_decode_micro_train_shard, CanaryFraudReport, ChunkVoucherDelivery, ComputeCompletionClaim,
    FountainDecoder, FountainDroplet, MicroBatchManifest, TaskId, WeightDeltaClaim, WireError,
    MAX_INLINE_DELTA_F16_LEN,
};
use thiserror::Error;

use crate::compute_flow::{ComputeFlowCoordinator, ComputeFlowError};
use crate::governor_fsm::{GovernorConfig, GovernorFsm, GovernorState, SystemTelemetrySnapshot};
use crate::workload_mux::{PowerProfitabilityGuard, WorkloadClass, WorkloadMultiplexer, WorkloadMuxError};

pub const MAX_IN_FLIGHT_SHARDS: usize = 32;
const MICRO_LOSS_SCALE: f32 = 1_000_000.0;

#[derive(Debug, Error)]
pub enum OpportunisticWorkerError {
    #[error("workload multiplexer error: {0}")]
    Mux(#[from] WorkloadMuxError),
    #[error("micro-training error: {0}")]
    MicroTrain(#[from] MicroTrainError),
    #[error("checkpoint error: {0}")]
    Checkpoint(#[from] CheckpointError),
    #[error("compute flow error: {0}")]
    Compute(#[from] ComputeFlowError),
    #[error("ledger error: {0}")]
    Ledger(#[from] LedgerError),
    #[error("escrow error: {0}")]
    Escrow(#[from] EscrowError),
    #[error("wire error: {0}")]
    Wire(#[from] WireError),
    #[error("droplet references an unaccepted shard object hash")]
    UnknownShardObject,
    #[error("reconstructed shard digest does not match manifest")]
    ShardHashMismatch,
    #[error("maximum concurrent in-flight shards ({MAX_IN_FLIGHT_SHARDS}) exceeded")]
    TooManyInFlightShards,
}

#[derive(Debug, Clone)]
pub enum ShardExecutionStep {
    Completed { result: MicroTrainResult, claim: Box<WeightDeltaClaim> },
    Preempted(MicroTrainCheckpoint),
}

/// Orchestrates opportunistic edge micro-training, `<10ms` inference, Canary auditing, and settlement.
pub struct OpportunisticWorkerEngine {
    pub governor: GovernorFsm,
    pub guard: PreemptGuard,
    pub mux: WorkloadMultiplexer,
    pub power_guard: PowerProfitabilityGuard,
    pub canary_cfg: CanaryConfig,
    pub sweep_cfg: Option<YieldSweepConfig>,
    pub completed_shards: u64,
    pub cumulative_earned_nano: NanoSkn,
    pending_manifests: HashMap<[u8; 32], MicroBatchManifest>,
    shard_decoders: HashMap<[u8; 32], FountainDecoder>,
    checkpoints: HashMap<(TaskId, u16), MicroTrainCheckpoint>,
    canary_references: HashMap<(TaskId, u16), MicroTrainResult>,
    voucher_trackers: HashMap<UtxoId, VoucherChannelTracker>,
    tx_nonce: u64,
}

impl Default for OpportunisticWorkerEngine {
    fn default() -> Self { Self::new(GovernorConfig::default(), PowerProfitabilityGuard::with_floor_nano_per_sec(0)) }
}

impl OpportunisticWorkerEngine {
    pub fn new(governor_cfg: GovernorConfig, power_guard: PowerProfitabilityGuard) -> Self {
        Self {
            governor: GovernorFsm::new(governor_cfg), guard: PreemptGuard::new(), mux: WorkloadMultiplexer::new(),
            power_guard, canary_cfg: CanaryConfig::default(), sweep_cfg: None, completed_shards: 0,
            cumulative_earned_nano: NanoSkn::ZERO, pending_manifests: HashMap::new(), shard_decoders: HashMap::new(),
            checkpoints: HashMap::new(), canary_references: HashMap::new(), voucher_trackers: HashMap::new(), tx_nonce: 0,
        }
    }

    pub fn with_sweep_config(mut self, sweep_cfg: YieldSweepConfig) -> Self { self.sweep_cfg = Some(sweep_cfg); self }
    pub fn governor_state(&self) -> GovernorState { self.governor.state() }
    pub fn update_telemetry(&mut self, snap: &SystemTelemetrySnapshot) -> GovernorState { self.governor.evaluate(snap, &self.guard) }

    pub fn config_from_manifest(m: &MicroBatchManifest) -> MicroTrainConfig {
        MicroTrainConfig {
            dim_in: usize::from(m.dim_in), dim_hidden: usize::from(m.dim_hidden), dim_out: usize::from(m.dim_out),
            inner_steps: m.inner_steps, learning_rate_micro: m.learning_rate_micro, max_grad_norm_milli: DEFAULT_MAX_GRAD_NORM_MILLI,
        }
    }

    pub fn accept_manifest(&mut self, manifest: MicroBatchManifest, est_duration_ms: u64) -> Result<(), OpportunisticWorkerError> {
        self.mux.ensure_admitted(self.governor.state(), WorkloadClass::MicroTrainingShard)?;
        self.power_guard.evaluate_bid(NanoSkn(manifest.bounty_per_chunk_nano), est_duration_ms)?;
        if !self.pending_manifests.contains_key(&manifest.shard_object_hash) && self.pending_manifests.len() >= MAX_IN_FLIGHT_SHARDS {
            return Err(OpportunisticWorkerError::TooManyInFlightShards);
        }
        self.pending_manifests.insert(manifest.shard_object_hash, manifest);
        Ok(())
    }

    pub fn has_pending_shard(&self, object_hash: &[u8; 32]) -> bool { self.pending_manifests.contains_key(object_hash) }

    pub fn absorb_shard_droplet(&mut self, droplet: FountainDroplet) -> Result<Option<(MicroBatchManifest, MicroTrainBatch)>, OpportunisticWorkerError> {
        if !self.pending_manifests.contains_key(&droplet.object_hash) { return Err(OpportunisticWorkerError::UnknownShardObject); }
        let hash = droplet.object_hash;
        let dec = self.shard_decoders.entry(hash).or_insert_with(|| FountainDecoder::from_first_droplet(&droplet));
        let Some(raw) = try_decode_micro_train_shard(dec, droplet)? else { return Ok(None); };
        self.shard_decoders.remove(&hash);
        let manifest = self.pending_manifests.remove(&hash).ok_or(OpportunisticWorkerError::UnknownShardObject)?;
        if *blake3::hash(&raw).as_bytes() != manifest.shard_object_hash { return Err(OpportunisticWorkerError::ShardHashMismatch); }
        Ok(Some((manifest, bincode::deserialize(&raw).map_err(WireError::Codec)?)))
    }

    pub fn run_or_pause_shard(&mut self, worker_key: &SigningKey, cfg: &MicroTrainConfig, batch: &MicroTrainBatch) -> Result<ShardExecutionStep, OpportunisticWorkerError> {
        if !WorkloadMultiplexer::allows_class(self.governor.state(), WorkloadClass::MicroTrainingShard) { self.guard.trigger_preempt(); }
        let key = (batch.task_id, batch.shard_id);
        match execute_micro_train_slices(cfg, batch, self.checkpoints.get(&key), &self.guard)? {
            MicroTrainOutcome::Preempted(ckpt) => { self.checkpoints.insert(key, ckpt.clone()); Ok(ShardExecutionStep::Preempted(ckpt)) }
            MicroTrainOutcome::Completed(result) => {
                self.checkpoints.remove(&key);
                self.completed_shards = self.completed_shards.saturating_add(1);
                let claim = Box::new(build_signed_delta_claim(worker_key, &result)?);
                Ok(ShardExecutionStep::Completed { result, claim })
            }
        }
    }

    pub fn checkpoint_for(&self, task_id: &TaskId, shard_id: u16) -> Option<&MicroTrainCheckpoint> { self.checkpoints.get(&(*task_id, shard_id)) }
    pub fn restore_checkpoint(&mut self, ckpt: MicroTrainCheckpoint) -> Result<(), OpportunisticWorkerError> { ckpt.verify_integrity()?; self.checkpoints.insert((ckpt.task_id, ckpt.shard_id), ckpt); Ok(()) }

    pub fn execute_fast_inference(&self, worker_key: &SigningKey, input: &CandleTaskInput) -> Result<(CandleTaskOutput, ComputeCompletionClaim), OpportunisticWorkerError> {
        self.mux.ensure_admitted(self.governor.state(), WorkloadClass::FastInferenceSub10ms)?;
        Ok(ComputeFlowCoordinator::new().execute_and_claim(worker_key, input)?)
    }

    pub fn register_canary_reference(&mut self, reference: MicroTrainResult) { self.canary_references.insert((reference.task_id, reference.shard_id), reference); }
    pub fn canary_reference(&self, task_id: &TaskId, shard_id: u16) -> Option<&MicroTrainResult> { self.canary_references.get(&(*task_id, shard_id)) }

    pub fn verify_claim_against_canary(&self, auditor_key: &SigningKey, claim: &WeightDeltaClaim, reference: &MicroTrainResult) -> Result<Option<CanaryFraudReport>, OpportunisticWorkerError> {
        self.mux.ensure_admitted(self.governor.state(), WorkloadClass::CanaryVerification)?;
        claim.verify_signature()?;
        let worker_res = MicroTrainResult {
            task_id: claim.task_id, shard_id: claim.shard_id, completed_steps: claim.completed_steps,
            delta_f16: claim.inline_delta_f16.clone(), updated_weights_f16: Vec::new(),
            initial_loss: (claim.initial_loss_micro as f32) / MICRO_LOSS_SCALE,
            final_loss: (claim.final_loss_micro as f32) / MICRO_LOSS_SCALE, canonical_digest: claim.canonical_digest,
        };
        let CanaryVerdict::FraudDetected { measured_cosine_milli, .. } = verify_canary_spot_check(&worker_res, reference, &self.canary_cfg) else { return Ok(None); };
        let rep = CanaryFraudReport {
            task_id: claim.task_id, shard_id: claim.shard_id, faulty_worker: claim.worker_pubkey, challenger_auditor: [0; 32],
            measured_cosine_milli, evidence_digest: claim.canonical_digest, auditor_signature: Vec::new(),
        };
        Ok(Some(rep.into_signed(auditor_key)))
    }

    pub fn apply_fraud_report_to_vault(vault: &mut ComputeEscrowVault, r: &CanaryFraudReport) -> Result<SlashOutcome, OpportunisticWorkerError> {
        r.verify_signature()?;
        Ok(vault.slash_on_fraud_proof(FraudProofClaim {
            task_id: r.task_id, faulty_worker: r.faulty_worker, challenger_auditor: r.challenger_auditor,
            measured_cosine_milli: r.measured_cosine_milli, evidence_digest: r.evidence_digest,
        })?)
    }

    pub fn receive_chunk_voucher(&mut self, delivery: ChunkVoucherDelivery, max_allowance: NanoSkn) -> Result<NanoSkn, OpportunisticWorkerError> {
        let tracker = self.voucher_trackers.entry(delivery.voucher.bond_utxo)
            .or_insert_with(|| VoucherChannelTracker::new(delivery.voucher.bond_utxo, max_allowance));
        let delta = tracker.receive_udp_voucher(delivery.voucher)?;
        self.cumulative_earned_nano = self.cumulative_earned_nano.checked_add(delta)?;
        Ok(delta)
    }

    pub fn tracker_for_bond(&self, bond_utxo: &UtxoId) -> Option<&VoucherChannelTracker> { self.voucher_trackers.get(bond_utxo) }

    pub fn consolidate_direct_to_bond_tx(&mut self, agent_key: &SigningKey, ledger: &UtxoDagLedger) -> Result<Option<SknTransaction>, OpportunisticWorkerError> {
        let Some(cfg) = &self.sweep_cfg else { return Ok(None); };
        let owned = ledger.utxos_for_owner(&agent_key.verifying_key().to_bytes());
        if !owned.iter().any(|(_, o)| matches!(o.policy, CustodyPolicy::DirectOwner)) { return Ok(None); }
        build_bond_consolidation_tx(agent_key, &owned, cfg.human_owner, &mut self.tx_nonce)
    }

    pub fn try_build_yield_sweep(&mut self, agent_key: &SigningKey, ledger: &UtxoDagLedger) -> Result<Option<SknTransaction>, OpportunisticWorkerError> {
        let Some(cfg) = &self.sweep_cfg else { return Ok(None); };
        let bonds: Vec<(UtxoId, TxOutput)> = ledger.utxos_for_owner(&agent_key.verifying_key().to_bytes()).into_iter()
            .filter(|(_, o)| matches!(o.policy, CustodyPolicy::AgentBond { human_owner, .. } if human_owner == cfg.human_owner))
            .collect();
        self.tx_nonce = self.tx_nonce.wrapping_add(1);
        Ok(build_yield_sweep_tx(agent_key, &bonds, cfg, self.tx_nonce)?)
    }
}

fn build_signed_delta_claim(worker_key: &SigningKey, res: &MicroTrainResult) -> Result<WeightDeltaClaim, OpportunisticWorkerError> {
    let (inline_delta_f16, delta_object_hash) = if res.delta_f16.len() <= MAX_INLINE_DELTA_F16_LEN {
        (res.delta_f16.clone(), None)
    } else {
        (Vec::new(), Some(*blake3::hash(&bincode::serialize(&res.delta_f16).map_err(WireError::Codec)?).as_bytes()))
    };
    Ok(WeightDeltaClaim {
        task_id: res.task_id, shard_id: res.shard_id, worker_pubkey: [0; 32], completed_steps: res.completed_steps,
        initial_loss_micro: (res.initial_loss.max(0.0) * MICRO_LOSS_SCALE).round() as u64,
        final_loss_micro: (res.final_loss.max(0.0) * MICRO_LOSS_SCALE).round() as u64,
        canonical_digest: res.canonical_digest, inline_delta_f16, delta_object_hash, worker_signature: Vec::new(),
    }.into_signed(worker_key)?)
}

fn build_bond_consolidation_tx(agent_key: &SigningKey, owned: &[(UtxoId, TxOutput)], human: [u8; 32], nonce: &mut u64) -> Result<Option<SknTransaction>, OpportunisticWorkerError> {
    let (mut inputs, mut total, mut allowance) = (Vec::new(), NanoSkn::ZERO, NanoSkn::ZERO);
    for (id, out) in owned {
        match out.policy {
            CustodyPolicy::DirectOwner => { total = total.checked_add(out.amount)?; inputs.push(*id); }
            CustodyPolicy::AgentBond { human_owner, max_voucher_allowance } if human_owner == human => {
                total = total.checked_add(out.amount)?; allowance = allowance.max(max_voucher_allowance); inputs.push(*id);
            }
            _ => {}
        }
    }
    if inputs.is_empty() { return Ok(None); }
    *nonce = nonce.wrapping_add(1);
    let parents = inputs.iter().map(|u| u.tx_hash).collect::<BTreeSet<_>>().into_iter().collect();
    let out = TxOutput::agent_bond(human, agent_key.verifying_key().to_bytes(), total, allowance);
    Ok(Some(SknTransaction::new_signed(agent_key, inputs, vec![out], NanoSkn::ZERO, parents, *nonce)))
}
