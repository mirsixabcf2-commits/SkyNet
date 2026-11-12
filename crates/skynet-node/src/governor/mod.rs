//! `governor`: Adaptive rootless OS telemetry governor (`250ms` active / `1500ms` paused)
//! driving `PreemptGuard` in `< 100ms` via `tokio::task::spawn_blocking`.

pub mod checkpoint_store;
pub mod probe_linux;
pub mod probe_macos;
pub mod probe_windows;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use skynet_agent::{GovernorConfig, GovernorFsm, GovernorState, SystemTelemetrySnapshot};
use skynet_cell::PreemptGuard;
use thiserror::Error;
use tokio::task::JoinHandle;

pub use checkpoint_store::{
    CheckpointStore, CheckpointStoreError, WorkerStudioStateSnapshot, WORKER_STUDIO_STATE_FILENAME,
};
pub use probe_linux::LinuxTelemetryProbe;
pub use probe_macos::MacOsTelemetryProbe;
pub use probe_windows::{SimulatedTelemetryProbe, WindowsTelemetryProbe};

/// Production AAA game & heavy DCC process watchlist triggering `< 100ms` preemption.
pub const DEFAULT_GAME_WATCHLIST: &[&str] = &[
    "cs2",
    "valorant",
    "dota2",
    "leagueclient",
    "cyberpunk2077",
    "eldenring",
    "genshinimpact",
    "baldur",
    "unreal",
    "blender",
];

#[derive(Debug, Error)]
pub enum GovernorError {
    #[error("blocking OS telemetry task failed: {0}")]
    BlockingJoin(#[from] tokio::task::JoinError),
    #[error("governor state mutex poisoned")]
    StateLockPoisoned,
}

#[derive(Debug, Clone)]
enum ProbeDriver {
    Host {
        macos: MacOsTelemetryProbe,
        linux: Arc<LinuxTelemetryProbe>,
        windows: WindowsTelemetryProbe,
    },
    Simulated(SimulatedTelemetryProbe),
}

impl ProbeDriver {
    fn sample_blocking(&self) -> SystemTelemetrySnapshot {
        let Self::Host { macos, linux, windows } = self else {
            return match self {
                Self::Simulated(sim) => sim.sample(),
                Self::Host { .. } => SystemTelemetrySnapshot::default(),
            };
        };
        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        match std::env::consts::OS {
            "macos" => macos.sample(now_sec),
            "linux" => linux.sample(now_sec),
            "windows" => windows.sample(now_sec),
            _ => SystemTelemetrySnapshot { observed_at_sec: now_sec, ..Default::default() },
        }
    }
}

struct GovernorShared {
    fsm: GovernorFsm,
    latest_snapshot: SystemTelemetrySnapshot,
    last_preempt_latency_ms: f64,
}

/// Adaptive background OS hardware governor driving `PreemptGuard` atomically.
#[derive(Clone)]
pub struct OsTelemetryGovernor {
    driver: ProbeDriver,
    guard: PreemptGuard,
    shared: Arc<Mutex<GovernorShared>>,
}

impl OsTelemetryGovernor {
    pub fn new_host(
        config: GovernorConfig,
        custom_watchlist: Vec<String>,
        guard: PreemptGuard,
    ) -> Self {
        let driver = ProbeDriver::Host {
            macos: MacOsTelemetryProbe::new(custom_watchlist.clone()),
            linux: Arc::new(LinuxTelemetryProbe::new(custom_watchlist.clone())),
            windows: WindowsTelemetryProbe::new(custom_watchlist),
        };
        Self::from_driver(config, driver, guard)
    }

    pub fn new_simulated(
        config: GovernorConfig,
        probe: SimulatedTelemetryProbe,
        guard: PreemptGuard,
    ) -> Self {
        Self::from_driver(config, ProbeDriver::Simulated(probe), guard)
    }

    fn from_driver(config: GovernorConfig, driver: ProbeDriver, guard: PreemptGuard) -> Self {
        let shared = GovernorShared {
            fsm: GovernorFsm::new(config),
            latest_snapshot: SystemTelemetrySnapshot::default(),
            last_preempt_latency_ms: 0.0,
        };
        Self { driver, guard, shared: Arc::new(Mutex::new(shared)) }
    }

    pub fn guard(&self) -> &PreemptGuard {
        &self.guard
    }

    pub fn current_state(&self) -> GovernorState {
        self.shared.lock().map(|g| g.fsm.state()).unwrap_or(GovernorState::DeepIdle)
    }

    pub fn latest_snapshot(&self) -> SystemTelemetrySnapshot {
        self.shared.lock().map(|g| g.latest_snapshot).unwrap_or_default()
    }

    pub fn last_preempt_latency_ms(&self) -> f64 {
        self.shared.lock().map(|g| g.last_preempt_latency_ms).unwrap_or(0.0)
    }

    /// Collects one telemetry sample inside `spawn_blocking` and updates `PreemptGuard`.
    pub async fn poll_once(
        &self,
    ) -> Result<(GovernorState, SystemTelemetrySnapshot), GovernorError> {
        let started = Instant::now();
        let driver = self.driver.clone();
        let snap = tokio::task::spawn_blocking(move || driver.sample_blocking()).await?;
        let mut guard = self.shared.lock().map_err(|_| GovernorError::StateLockPoisoned)?;
        let prev = guard.fsm.state();
        let next = guard.fsm.evaluate(&snap, &self.guard);
        guard.latest_snapshot = snap;
        if !prev.is_paused() && next.is_paused() {
            guard.last_preempt_latency_ms = started.elapsed().as_secs_f64() * 1_000.0;
        }
        Ok((next, snap))
    }

    /// Spawns the adaptive `250ms` (active) / `1500ms` (paused) background telemetry loop.
    pub fn spawn_loop(&self, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
        let gov = self.clone();
        let sim_wake = match &self.driver {
            ProbeDriver::Simulated(s) => Some(s.wake_notifier()),
            ProbeDriver::Host { .. } => None,
        };
        tokio::spawn(async move {
            while !stop_flag.load(Ordering::Relaxed) {
                let poll_ms = gov
                    .poll_once()
                    .await
                    .map(|(st, _)| st.recommended_poll_ms())
                    .unwrap_or(250);
                wait_next_poll(poll_ms, sim_wake.as_ref()).await;
            }
        })
    }
}

async fn wait_next_poll(poll_ms: u64, sim_wake: Option<&Arc<tokio::sync::Notify>>) {
    let sleep_fut = tokio::time::sleep(Duration::from_millis(poll_ms));
    match sim_wake {
        Some(notify) => {
            tokio::select! {
                _ = sleep_fut => {}
                _ = notify.notified() => {}
            }
        }
        None => sleep_fut.await,
    }
}

/// Case-insensitive matcher checking `process_text` against `DEFAULT_GAME_WATCHLIST` + `custom`.
pub fn matches_game_watchlist(process_text: &str, custom: &[String]) -> bool {
    if process_text.trim().is_empty() {
        return false;
    }
    let lower = process_text.to_lowercase();
    if DEFAULT_GAME_WATCHLIST.iter().any(|g| lower.contains(g)) {
        return true;
    }
    custom
        .iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .any(|g| lower.contains(&g))
}
