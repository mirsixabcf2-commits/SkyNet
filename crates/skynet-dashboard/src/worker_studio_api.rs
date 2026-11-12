//! `worker_studio_api`: Worker Earnings & Opportunistic Studio telemetry controller,
//! optional live `worker_studio_state.json` disk loader, and interactive simulation triggers.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use ed25519_dalek::SigningKey;
use skynet_agent::{GovernorConfig, GovernorFsm, SystemTelemetrySnapshot};
use skynet_cell::{MicroTrainCheckpoint, PreemptGuard};
use skynet_ledger::compute_escrow::ComputeEscrowVault;
use skynet_ledger::{
    plan_yield_sweep, FraudProofClaim, HumanWallet, NanoSkn, TxOutput, UtxoId, YieldSweepConfig,
    DEFAULT_AUTO_SWEEP_NANO_SKN, NANO_PER_SKN,
};

use crate::dto::WorkerStudioTelemetryDto;
use crate::world_actions::resolve_skynet_data_dir;

pub const WORKER_STUDIO_STATE_FILE: &str = "worker_studio_state.json";
const SIM_HOLD_WINDOW_SEC: u64 = 30;
const CHUNK_EARN_STEP_NANO: u128 = 250_000_000;
const MIN_SWEEP_DEMO_YIELD_NANO: u128 = DEFAULT_AUTO_SWEEP_NANO_SKN + 5 * NANO_PER_SKN;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioAction { SimGameLaunch, SimThermalSpike, SimCanaryFraud, SimYieldSweep, SimResumeIdle }

impl StudioAction {
    pub fn parse(action: &str) -> Option<Self> {
        match action {
            "sim_game_launch" => Some(Self::SimGameLaunch),
            "sim_thermal_spike" => Some(Self::SimThermalSpike),
            "sim_canary_fraud" => Some(Self::SimCanaryFraud),
            "sim_yield_sweep" => Some(Self::SimYieldSweep),
            "sim_resume_idle" => Some(Self::SimResumeIdle),
            _ => None,
        }
    }
}

/// Reads `worker_studio_state.json` from `custom_path`, `SKYNET_WORKER_STUDIO_STATE`, or data dir.
pub fn load_worker_studio_state_file(custom_path: Option<&Path>) -> Option<WorkerStudioTelemetryDto> {
    let resolved: PathBuf = match custom_path {
        Some(p) => p.to_path_buf(),
        None => std::env::var("SKYNET_WORKER_STUDIO_STATE").map(PathBuf::from).unwrap_or_else(|_| resolve_skynet_data_dir().join(WORKER_STUDIO_STATE_FILE)),
    };
    serde_json::from_str::<WorkerStudioTelemetryDto>(&std::fs::read_to_string(resolved).ok()?).ok()
}

/// Stateful Studio controller backed by real `GovernorFsm`, `ComputeEscrowVault`, and `YieldSweep`.
pub struct WorkerStudioController {
    fsm: GovernorFsm, guard: PreemptGuard, escrow: ComputeEscrowVault,
    human_owner: HumanWallet, worker_key: SigningKey, operating_reserve: NanoSkn,
    tx_nonce: u64, unswept_yield_nano: u128, override_until_sec: u64,
    telemetry: WorkerStudioTelemetryDto, sweep_log: VecDeque<String>,
}

impl Default for WorkerStudioController { fn default() -> Self { Self::new() } }

impl WorkerStudioController {
    pub fn new() -> Self {
        let human_owner = HumanWallet::from_recovery_phrase("studio-gamer-cold-wallet-2026");
        let (worker_key, _) = human_owner.authorize_agent(0, NanoSkn(20 * NANO_PER_SKN), 100_000);
        let mut telemetry = WorkerStudioTelemetryDto { updated_at_sec: now_unix_sec(), ..Default::default() };
        if let Some(disk) = load_worker_studio_state_file(None) { telemetry = disk; }
        Self {
            fsm: GovernorFsm::new(GovernorConfig::default()), guard: PreemptGuard::new(),
            escrow: ComputeEscrowVault::new(), human_owner, worker_key,
            operating_reserve: NanoSkn(100 * NANO_PER_SKN), tx_nonce: 0, unswept_yield_nano: 0,
            override_until_sec: 0, telemetry, sweep_log: VecDeque::new(),
        }
    }

    pub fn refresh_from_disk(&mut self, custom_path: Option<&Path>) {
        if now_unix_sec() < self.override_until_sec { return; }
        if let Some(disk) = load_worker_studio_state_file(custom_path) { self.telemetry = disk; }
    }

    pub fn tick_opportunistic(&mut self) {
        let now = now_unix_sec();
        if now < self.override_until_sec { self.telemetry.updated_at_sec = now; return; }
        if let Some(disk) = load_worker_studio_state_file(None) { self.telemetry = disk; return; }
        self.telemetry.fountain_shards_completed = self.telemetry.fountain_shards_completed.saturating_add(1);
        self.telemetry.earned_nano_skn = self.telemetry.earned_nano_skn.saturating_add(CHUNK_EARN_STEP_NANO);
        self.unswept_yield_nano = self.unswept_yield_nano.saturating_add(CHUNK_EARN_STEP_NANO);
        self.telemetry.updated_at_sec = now;
    }

    pub fn apply_action(&mut self, action: StudioAction) -> String {
        let now = now_unix_sec();
        self.override_until_sec = now.saturating_add(SIM_HOLD_WINDOW_SEC);
        match action {
            StudioAction::SimGameLaunch => self.trigger_game_launch(now),
            StudioAction::SimThermalSpike => self.trigger_thermal_spike(now),
            StudioAction::SimCanaryFraud => self.trigger_canary_fraud(now),
            StudioAction::SimYieldSweep => self.trigger_yield_sweep(now),
            StudioAction::SimResumeIdle => self.resume_deep_idle(now),
        }
    }

    pub fn telemetry_dto(&self) -> WorkerStudioTelemetryDto { self.telemetry.clone() }
    pub fn sweep_log_vec(&self) -> Vec<String> { self.sweep_log.iter().cloned().collect() }
    pub fn telemetry_json(&self) -> String { serde_json::to_string(&self.telemetry).unwrap_or_else(|_| "{}".to_string()) }

    fn trigger_game_launch(&mut self, now: u64) -> String {
        let t0 = Instant::now();
        let snap = SystemTelemetrySnapshot { gpu_util_pct: 95, cpu_util_pct: 72, vram_used_pct: 74, temp_c: 66, on_battery: false, active_game_detected: true, observed_at_sec: now };
        let st = self.fsm.evaluate(&snap, &self.guard);
        let _ckpt = MicroTrainCheckpoint { task_id: [0xA1; 32], shard_id: 0, completed_steps: 3, total_steps: 8, weights_f16_bits: vec![0x3C00; 32], last_loss_micro: 420_000, checkpoint_digest: [0; 32] }.seal();
        let ms = (t0.elapsed().as_secs_f64() * 1_000.0).max(0.08);
        self.telemetry.governor_state = format!("{st:?}");
        self.telemetry.preempt_latency_ms = ms;
        self.telemetry.active_game_detected = true;
        self.telemetry.gpu_util_pct = 95; self.telemetry.cpu_util_pct = 72; self.telemetry.vram_used_pct = 74; self.telemetry.temp_c = 66;
        self.telemetry.earning_rate_skn_per_hr = 0.0; self.telemetry.updated_at_sec = now;
        format!("[GOVERNOR] Phát hiện Game AAA — Nhường 100% GPU trong {ms:.2}ms (<100ms SLA) & lưu Micro-Checkpoint!")
    }

    fn trigger_thermal_spike(&mut self, now: u64) -> String {
        let t0 = Instant::now();
        let snap = SystemTelemetrySnapshot { gpu_util_pct: 80, cpu_util_pct: 55, vram_used_pct: 48, temp_c: 82, on_battery: false, active_game_detected: false, observed_at_sec: now };
        let st = self.fsm.evaluate(&snap, &self.guard);
        let ms = (t0.elapsed().as_secs_f64() * 1_000.0).max(0.08);
        self.telemetry.governor_state = format!("{st:?}");
        self.telemetry.preempt_latency_ms = ms; self.telemetry.active_game_detected = false;
        self.telemetry.temp_c = 82; self.telemetry.gpu_util_pct = 80;
        self.telemetry.earning_rate_skn_per_hr = 0.0; self.telemetry.updated_at_sec = now;
        "[THERMAL] Nhiệt độ chạm 82°C (>= 78°C) — Tạm dừng bảo vệ phần cứng (chỉ nối lại khi <= 70°C & rảnh 30s)!".into()
    }

    fn trigger_canary_fraud(&mut self, now: u64) -> String {
        self.tx_nonce = self.tx_nonce.wrapping_add(1);
        let mut task_id = [0xCF; 32];
        task_id[0..8].copy_from_slice(&self.tx_nonce.to_le_bytes());
        let w_pk = self.worker_key.verifying_key().to_bytes();
        let _ = self.escrow.lock_task_simple(task_id, [0x11; 32], w_pk, NanoSkn(50_000), NanoSkn(25_000), 100);
        if let Ok(out) = self.escrow.slash_on_fraud_proof(FraudProofClaim { task_id, faulty_worker: w_pk, challenger_auditor: [0xAA; 32], measured_cosine_milli: -850, evidence_digest: [0xEE; 32] }) {
            self.telemetry.pouw_score_bps = out.updated_reputation.pouw_score_bps;
            self.telemetry.slashed_count = out.updated_reputation.slashed_count;
        }
        self.telemetry.updated_at_sec = now;
        format!("[CANARY PoUW] Bắt giữ Worker gian lận (Cosine < 0.985) — Cắt 100% cọc (slashed #{}) & hạ PoUW = 0 bps!", self.telemetry.slashed_count)
    }

    fn trigger_yield_sweep(&mut self, now: u64) -> String {
        if self.unswept_yield_nano < MIN_SWEEP_DEMO_YIELD_NANO {
            let top_up = MIN_SWEEP_DEMO_YIELD_NANO - self.unswept_yield_nano;
            self.unswept_yield_nano = MIN_SWEEP_DEMO_YIELD_NANO;
            self.telemetry.earned_nano_skn = self.telemetry.earned_nano_skn.saturating_add(top_up);
        }
        self.tx_nonce = self.tx_nonce.wrapping_add(1);
        let owner_pk = self.human_owner.human_pubkey();
        let w_pk = self.worker_key.verifying_key().to_bytes();
        let bond_out = TxOutput::agent_bond(owner_pk, w_pk, NanoSkn(self.operating_reserve.0.saturating_add(self.unswept_yield_nano)), NanoSkn(20 * NANO_PER_SKN));
        let cfg = YieldSweepConfig::new(owner_pk, self.operating_reserve);
        if let Ok(Some(order)) = plan_yield_sweep(&self.worker_key, &[(UtxoId { tx_hash: [0x77; 32], vout: 0 }, bond_out)], &cfg, self.tx_nonce) {
            self.telemetry.swept_to_cold_nano_skn = self.telemetry.swept_to_cold_nano_skn.saturating_add(order.swept_yield.0);
            self.unswept_yield_nano = 0;
            let entry = format!("Tx #{}: +{:.3} SKN -> Ví Lạnh Chủ Nhân (0% phí burn)", self.tx_nonce, (order.swept_yield.0 as f64) / (NANO_PER_SKN as f64));
            self.sweep_log.push_front(entry.clone());
            while self.sweep_log.len() > 12 { self.sweep_log.pop_back(); }
            self.telemetry.updated_at_sec = now;
            return format!("[YIELD SWEEP] {entry}");
        }
        "[YIELD SWEEP] Số dư chưa vượt ngưỡng quét tự động 50 SKN".into()
    }

    fn resume_deep_idle(&mut self, now: u64) -> String {
        self.override_until_sec = 0;
        self.fsm = GovernorFsm::new(GovernorConfig::default());
        self.guard.reset_run();
        self.telemetry.governor_state = "DeepIdle".into();
        self.telemetry.active_game_detected = false; self.telemetry.temp_c = 48; self.telemetry.gpu_util_pct = 14;
        self.telemetry.earning_rate_skn_per_hr = 14.5; self.telemetry.updated_at_sec = now;
        "[GOVERNOR] Máy đã nguội & rảnh đủ 30s — Tiếp tục Micro-Training từ Checkpoint ở trạng thái DeepIdle!".into()
    }
}

fn now_unix_sec() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn studio_actions_and_disk_state_roundtrip() {
        let mut ctrl = WorkerStudioController::new();
        ctrl.apply_action(StudioAction::SimGameLaunch);
        assert_eq!(ctrl.telemetry_dto().governor_state, "PausedGaming");
        ctrl.apply_action(StudioAction::SimThermalSpike);
        assert_eq!(ctrl.telemetry_dto().governor_state, "PausedThermal");
        ctrl.apply_action(StudioAction::SimCanaryFraud);
        assert_eq!((ctrl.telemetry_dto().pouw_score_bps, ctrl.telemetry_dto().slashed_count), (0, 1));
        ctrl.apply_action(StudioAction::SimYieldSweep);
        assert!(ctrl.telemetry_dto().swept_to_cold_nano_skn >= DEFAULT_AUTO_SWEEP_NANO_SKN);

        let path = std::env::temp_dir().join("skynet_test_worker_studio_state.json");
        std::fs::write(&path, ctrl.telemetry_json()).expect("write state json");
        let loaded = load_worker_studio_state_file(Some(&path)).expect("load state json");
        let _ = std::fs::remove_file(&path);
        assert_eq!(loaded, ctrl.telemetry_dto());
    }
}
