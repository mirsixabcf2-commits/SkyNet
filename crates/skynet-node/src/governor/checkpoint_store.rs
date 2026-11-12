//! `checkpoint_store`: Atomic `.tmp -> rename` (`< 5ms`) persistence for `MicroTrainCheckpoint`
//! and canonical `worker_studio_state.json` telemetry export for `skynet-dashboard`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use skynet_cell::{CheckpointError, MicroTrainCheckpoint};
use thiserror::Error;

pub const WORKER_STUDIO_STATE_FILENAME: &str = "worker_studio_state.json";
static TMP_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Error)]
pub enum CheckpointStoreError {
    #[error("checkpoint filesystem I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("checkpoint binary codec error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("studio state JSON codec error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("checkpoint digest integrity error: {0}")]
    Integrity(#[from] CheckpointError),
}

/// Canonical JSON schema for `worker_studio_state.json` shared with `skynet-dashboard`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkerStudioStateSnapshot {
    pub governor_state: String,
    pub preempt_latency_ms: f64,
    pub gpu_util_pct: u8,
    pub cpu_util_pct: u8,
    pub vram_used_pct: u8,
    pub temp_c: u8,
    pub on_battery: bool,
    pub active_game_detected: bool,
    pub fountain_shards_completed: u64,
    pub pouw_score_bps: u16,
    pub slashed_count: u32,
    pub earned_nano_skn: u128,
    pub swept_to_cold_nano_skn: u128,
    pub earning_rate_skn_per_hr: f64,
    pub updated_at_sec: u64,
}

impl Default for WorkerStudioStateSnapshot {
    fn default() -> Self {
        Self {
            governor_state: "DeepIdle".to_string(),
            preempt_latency_ms: 0.0,
            gpu_util_pct: 5,
            cpu_util_pct: 5,
            vram_used_pct: 15,
            temp_c: 45,
            on_battery: false,
            active_game_detected: false,
            fountain_shards_completed: 0,
            pouw_score_bps: 5_000,
            slashed_count: 0,
            earned_nano_skn: 0,
            swept_to_cold_nano_skn: 0,
            earning_rate_skn_per_hr: 0.0,
            updated_at_sec: 0,
        }
    }
}

/// Crash-safe atomic disk store for preempted micro-training checkpoints and Studio telemetry.
#[derive(Debug, Clone)]
pub struct CheckpointStore {
    base_dir: PathBuf,
}

impl CheckpointStore {
    pub fn new(base_dir: impl Into<PathBuf>) -> Result<Self, CheckpointStoreError> {
        let base_dir = base_dir.into();
        fs::create_dir_all(&base_dir)?;
        Ok(Self { base_dir })
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn studio_state_path(&self) -> PathBuf {
        self.base_dir.join(WORKER_STUDIO_STATE_FILENAME)
    }

    /// Writes `ckpt` atomically via `.tmp -> rename` after verifying `BLAKE3` integrity.
    pub fn save_checkpoint(
        &self,
        ckpt: &MicroTrainCheckpoint,
    ) -> Result<PathBuf, CheckpointStoreError> {
        ckpt.verify_integrity()?;
        let bytes = bincode::serialize(ckpt)?;
        let final_path = self.checkpoint_path(&ckpt.task_id, ckpt.shard_id);
        write_bytes_atomic(&final_path, &bytes)?;
        Ok(final_path)
    }

    /// Loads and verifies a previously saved `MicroTrainCheckpoint`, if present on disk.
    pub fn load_checkpoint(
        &self,
        task_id: &[u8; 32],
        shard_id: u16,
    ) -> Result<Option<MicroTrainCheckpoint>, CheckpointStoreError> {
        let path = self.checkpoint_path(task_id, shard_id);
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read(path)?;
        let ckpt: MicroTrainCheckpoint = bincode::deserialize(&raw)?;
        ckpt.verify_integrity()?;
        Ok(Some(ckpt))
    }

    /// Removes a completed shard's checkpoint file if it exists.
    pub fn remove_checkpoint(
        &self,
        task_id: &[u8; 32],
        shard_id: u16,
    ) -> Result<(), CheckpointStoreError> {
        let path = self.checkpoint_path(task_id, shard_id);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Atomically publishes `worker_studio_state.json` via `.tmp -> rename`.
    pub fn write_studio_state(
        &self,
        state: &WorkerStudioStateSnapshot,
    ) -> Result<PathBuf, CheckpointStoreError> {
        let final_path = self.studio_state_path();
        let json = serde_json::to_vec(state)?;
        write_bytes_atomic(&final_path, &json)?;
        Ok(final_path)
    }

    /// Reads `worker_studio_state.json` if present.
    pub fn load_studio_state(
        &self,
    ) -> Result<Option<WorkerStudioStateSnapshot>, CheckpointStoreError> {
        let path = self.studio_state_path();
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(path)?;
        Ok(Some(serde_json::from_str(&raw)?))
    }

    fn checkpoint_path(&self, task_id: &[u8; 32], shard_id: u16) -> PathBuf {
        let prefix: String = task_id[..8].iter().map(|b| format!("{b:02x}")).collect();
        self.base_dir.join(format!("ckpt_{prefix}_s{shard_id}.bin"))
    }
}

fn write_bytes_atomic(final_path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    let seq = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = final_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("state.bin");
    let tmp_path = final_path.with_file_name(format!(".{name}.{}.{seq}.tmp", std::process::id()));
    fs::write(&tmp_path, bytes)?;
    fs::rename(&tmp_path, final_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_atomic_checkpoint_under_5ms_and_studio_state_schema() {
        let dir = std::env::temp_dir().join(format!("skynet_ckpt_{}", rand::random::<u64>()));
        let store = CheckpointStore::new(&dir).unwrap();
        let ckpt = MicroTrainCheckpoint {
            task_id: [0xAB; 32],
            shard_id: 3,
            completed_steps: 4,
            total_steps: 10,
            weights_f16_bits: vec![0x3C00; 512],
            last_loss_micro: 240_000,
            checkpoint_digest: [0u8; 32],
        }
        .seal();

        let min_save = (0..3)
            .map(|_| {
                let start = Instant::now();
                store.save_checkpoint(&ckpt).unwrap();
                start.elapsed()
            })
            .min()
            .unwrap();
        assert!(min_save < std::time::Duration::from_millis(5));

        let loaded = store.load_checkpoint(&[0xAB; 32], 3).unwrap().unwrap();
        assert_eq!(loaded, ckpt);

        let studio = WorkerStudioStateSnapshot { governor_state: "PausedGaming".into(), temp_c: 67, ..Default::default() };
        let studio_path = store.write_studio_state(&studio).unwrap();
        let raw_val: serde_json::Value = serde_json::from_str(&fs::read_to_string(studio_path).unwrap()).unwrap();
        assert!(raw_val.get("earning_rate_skn_per_hr").is_some());
        assert_eq!(store.load_studio_state().unwrap().unwrap(), studio);

        let _ = fs::remove_dir_all(dir);
    }
}
