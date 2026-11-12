//! Integration tests verifying `< 100ms` preemption SLA on game launch / `78°C` thermal /
//! battery unplug, `< 5ms` atomic checkpoint persistence, `75°C` / `< 30s` anti-flap hysteresis,
//! and exact resume at `<= 70°C` + `>= 30s` (`T029`).

use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use skynet_agent::{GovernorConfig, GovernorState, SystemTelemetrySnapshot};
use skynet_cell::{
    encode_dense_f16, execute_micro_train_slices, MicroTrainBatch, MicroTrainCheckpoint,
    MicroTrainConfig, MicroTrainOutcome, PreemptGuard, PreemptSignal,
};
use skynet_node::{
    CheckpointStore, OsTelemetryGovernor, SimulatedTelemetryProbe, WorkerStudioStateSnapshot,
};

fn build_heavy_batch(cfg: &MicroTrainConfig) -> MicroTrainBatch {
    let w_len = cfg.expected_weights_len();
    let init_w: Vec<f32> = (0..w_len).map(|i| ((i as f32) * 0.017).sin() * 0.25).collect();
    let samples_x: Vec<f32> = (0..8 * cfg.dim_in).map(|i| ((i as f32) * 0.11).cos()).collect();
    let targets_y: Vec<f32> = (0..8 * cfg.dim_out).map(|i| ((i as f32) * 0.07).sin() * 0.4).collect();
    MicroTrainBatch {
        task_id: [0x5A; 32],
        shard_id: 1,
        base_weights_f16: encode_dense_f16(&init_w),
        samples_x,
        targets_y,
    }
}

fn assert_checkpoint_and_studio_persist(
    store: &CheckpointStore,
    ckpt: &MicroTrainCheckpoint,
    preempt_elapsed: Duration,
    state: GovernorState,
) {
    let min_save = (0..3)
        .map(|_| {
            let t0 = Instant::now();
            store.save_checkpoint(ckpt).expect("atomic checkpoint write");
            t0.elapsed()
        })
        .min()
        .expect("min save duration");
    assert!(min_save < Duration::from_millis(5), "Checkpoint write exceeded 5ms: {min_save:?}");

    let studio = WorkerStudioStateSnapshot {
        governor_state: format!("{state:?}"),
        preempt_latency_ms: preempt_elapsed.as_secs_f64() * 1_000.0,
        active_game_detected: true,
        ..Default::default()
    };
    store.write_studio_state(&studio).expect("write studio state");
    assert_eq!(store.load_studio_state().unwrap().unwrap().governor_state, "PausedGaming");
}

#[tokio::test]
async fn test_game_launch_preempts_under_100ms_and_saves_atomic_checkpoint_under_5ms() {
    let dir = std::env::temp_dir().join(format!("skynet_gov_sla_{}", rand::random::<u64>()));
    let store = CheckpointStore::new(&dir).expect("create checkpoint store");
    let sim = SimulatedTelemetryProbe::new(SystemTelemetrySnapshot { observed_at_sec: 10, ..Default::default() });
    let guard = PreemptGuard::new();
    let gov = OsTelemetryGovernor::new_simulated(GovernorConfig::default(), sim.clone(), guard.clone());
    let stop = Arc::new(AtomicBool::new(false));
    let loop_handle = gov.spawn_loop(Arc::clone(&stop));

    let cfg = MicroTrainConfig {
        dim_in: 16, dim_hidden: 32, dim_out: 8, inner_steps: 25_000,
        learning_rate_micro: 10_000, max_grad_norm_milli: 5_000,
    };
    let (train_cfg, train_batch, train_guard) = (cfg.clone(), build_heavy_batch(&cfg), guard.clone());
    let train_handle = tokio::task::spawn_blocking(move || {
        execute_micro_train_slices(&train_cfg, &train_batch, None, &train_guard)
    });

    tokio::time::sleep(Duration::from_millis(4)).await;
    let preempt_t0 = Instant::now();
    sim.update(|s| { s.active_game_detected = true; s.observed_at_sec = 11; });

    let outcome = train_handle.await.expect("join train").expect("micro train slices");
    let preempt_elapsed = preempt_t0.elapsed();
    assert!(preempt_elapsed < Duration::from_millis(100), "Preemption SLA breached: {preempt_elapsed:?}");
    let MicroTrainOutcome::Preempted(ckpt) = outcome else {
        panic!("expected Preempted checkpoint when AAA game launched");
    };
    assert!(ckpt.completed_steps < cfg.inner_steps);
    assert_eq!(gov.current_state(), GovernorState::PausedGaming);
    assert_checkpoint_and_studio_persist(&store, &ckpt, preempt_elapsed, gov.current_state());

    stop.store(true, Ordering::Relaxed);
    sim.update(|_| {});
    let _ = loop_handle.await;
    let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
async fn test_thermal_78c_battery_unplug_hysteresis_75c_hold_and_resume_at_70c_30s() {
    let dir = std::env::temp_dir().join(format!("skynet_gov_hyst_{}", rand::random::<u64>()));
    let store = CheckpointStore::new(&dir).expect("store");
    let sim = SimulatedTelemetryProbe::new(SystemTelemetrySnapshot { observed_at_sec: 100, ..Default::default() });
    let guard = PreemptGuard::new();
    let gov = OsTelemetryGovernor::new_simulated(GovernorConfig::default(), sim.clone(), guard.clone());

    let t_batt = Instant::now();
    sim.set_snapshot(SystemTelemetrySnapshot { on_battery: true, observed_at_sec: 105, ..Default::default() });
    assert_eq!(gov.poll_once().await.unwrap().0, GovernorState::PausedBattery);
    assert!(t_batt.elapsed() < Duration::from_millis(100));

    let t_therm = Instant::now();
    sim.set_snapshot(SystemTelemetrySnapshot { temp_c: 78, observed_at_sec: 110, ..Default::default() });
    assert_eq!((gov.poll_once().await.unwrap().0, guard.signal()), (GovernorState::PausedThermal, PreemptSignal::PreemptImmediate));
    assert!(t_therm.elapsed() < Duration::from_millis(100));

    // Anti-flap hold: 75°C (> 70°C ceiling) or < 30s at 70°C stays PausedThermal
    sim.set_snapshot(SystemTelemetrySnapshot { temp_c: 75, observed_at_sec: 150, ..Default::default() });
    assert_eq!(gov.poll_once().await.unwrap().0, GovernorState::PausedThermal);
    sim.set_snapshot(SystemTelemetrySnapshot { temp_c: 70, observed_at_sec: 160, ..Default::default() });
    assert_eq!(gov.poll_once().await.unwrap().0, GovernorState::PausedThermal);
    sim.set_snapshot(SystemTelemetrySnapshot { temp_c: 70, observed_at_sec: 185, ..Default::default() });
    assert_eq!(gov.poll_once().await.unwrap().0, GovernorState::PausedThermal);

    // At t=190s (30s continuous <= 70°C), resumes DeepIdle & completes checkpointed training
    sim.set_snapshot(SystemTelemetrySnapshot { temp_c: 70, observed_at_sec: 190, ..Default::default() });
    assert_eq!((gov.poll_once().await.unwrap().0, guard.signal()), (GovernorState::DeepIdle, PreemptSignal::Run));
    verify_checkpoint_resume_loss_decreases(&store, &guard);
    let _ = fs::remove_dir_all(dir);
}

fn verify_checkpoint_resume_loss_decreases(store: &CheckpointStore, guard: &PreemptGuard) {
    let cfg = MicroTrainConfig { dim_in: 8, dim_hidden: 16, dim_out: 4, inner_steps: 10, learning_rate_micro: 25_000, max_grad_norm_milli: 5_000 };
    let batch = build_heavy_batch(&cfg);
    let partial_guard = PreemptGuard::with_signal(PreemptSignal::PreemptImmediate);
    let MicroTrainOutcome::Preempted(ckpt) = execute_micro_train_slices(&cfg, &batch, None, &partial_guard).unwrap() else {
        panic!("expected immediate checkpoint");
    };
    store.save_checkpoint(&ckpt).unwrap();
    let loaded = store.load_checkpoint(&batch.task_id, batch.shard_id).unwrap().unwrap();
    let MicroTrainOutcome::Completed(res) = execute_micro_train_slices(&cfg, &batch, Some(&loaded), guard).unwrap() else {
        panic!("expected Completed on resume");
    };
    assert_eq!(res.completed_steps, cfg.inner_steps);
    assert!(res.final_loss < res.initial_loss);
}
