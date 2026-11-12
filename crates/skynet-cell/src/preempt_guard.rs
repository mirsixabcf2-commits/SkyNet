//! Zero-impact preemption primitives (`PreemptGuard`, `SliceLatencyGuard`) and
//! integrity-sealed `MicroTrainCheckpoint` for `< 100ms` opportunistic yielding.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SLICE_WARMUP_COUNT: u32 = 3;
pub const CONTENTION_SPIKE_NUM: u64 = 18;
pub const CONTENTION_SPIKE_DEN: u64 = 10;
pub const CHECKPOINT_DIGEST_DOMAIN: &[u8] = b"SKYNET_MICRO_CHECKPOINT_V1";

/// Atomic preemption signal shared lock-free between OS Governor and Candle worker slices.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreemptSignal {
    Run = 0,
    Throttle50 = 1,
    PreemptImmediate = 2,
}

impl PreemptSignal {
    pub fn from_u8(raw: u8) -> Self {
        match raw {
            0 => Self::Run,
            1 => Self::Throttle50,
            _ => Self::PreemptImmediate,
        }
    }
}

/// Lock-free `Arc<AtomicU8>` handle checked between micro-training slices.
#[derive(Debug, Clone)]
pub struct PreemptGuard {
    state: Arc<AtomicU8>,
}

impl Default for PreemptGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl PreemptGuard {
    pub fn new() -> Self {
        Self::with_signal(PreemptSignal::Run)
    }

    pub fn with_signal(initial: PreemptSignal) -> Self {
        Self { state: Arc::new(AtomicU8::new(initial as u8)) }
    }

    pub fn signal(&self) -> PreemptSignal {
        PreemptSignal::from_u8(self.state.load(Ordering::Acquire))
    }

    pub fn set_signal(&self, signal: PreemptSignal) {
        self.state.store(signal as u8, Ordering::Release);
    }

    pub fn trigger_preempt(&self) {
        self.set_signal(PreemptSignal::PreemptImmediate);
    }

    pub fn trigger_throttle(&self) {
        self.set_signal(PreemptSignal::Throttle50);
    }

    pub fn reset_run(&self) {
        self.set_signal(PreemptSignal::Run);
    }

    pub fn should_preempt(&self) -> bool {
        self.signal() == PreemptSignal::PreemptImmediate
    }

    pub fn is_throttled(&self) -> bool {
        self.signal() == PreemptSignal::Throttle50
    }
}

/// Syscall-free kernel contention detector triggering `PreemptImmediate` when a slice's
/// latency exceeds `1.8x` (`18 / 10`) of its EMA after 3 warmup slices.
#[derive(Debug, Clone, Default)]
pub struct SliceLatencyGuard {
    ema_slice_micros: u64,
    observed_slices: u32,
}

impl SliceLatencyGuard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ema_slice_micros(&self) -> u64 {
        self.ema_slice_micros
    }

    pub fn observed_slices(&self) -> u32 {
        self.observed_slices
    }

    pub fn reset(&mut self) {
        self.ema_slice_micros = 0;
        self.observed_slices = 0;
    }

    pub fn observe_slice(&mut self, actual_micros: u64, guard: &PreemptGuard) -> PreemptSignal {
        let sample = actual_micros.max(1);
        if self.observed_slices < SLICE_WARMUP_COUNT {
            self.ema_slice_micros = if self.observed_slices == 0 {
                sample
            } else {
                (self.ema_slice_micros.saturating_mul(3).saturating_add(sample)) / 4
            };
            self.observed_slices = self.observed_slices.saturating_add(1);
            return guard.signal();
        }

        self.observed_slices = self.observed_slices.saturating_add(1);
        let spike_limit = self.ema_slice_micros.saturating_mul(CONTENTION_SPIKE_NUM) / CONTENTION_SPIKE_DEN;
        if self.ema_slice_micros > 0 && actual_micros > spike_limit {
            guard.trigger_preempt();
            return PreemptSignal::PreemptImmediate;
        }
        self.ema_slice_micros = (self.ema_slice_micros.saturating_mul(3).saturating_add(sample)) / 4;
        guard.signal()
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CheckpointError {
    #[error("checkpoint BLAKE3 digest mismatch")]
    DigestMismatch,
    #[error("invalid step progress: completed {completed} > total {total}")]
    InvalidStepProgress { completed: u16, total: u16 },
    #[error("checkpoint weights buffer is empty")]
    EmptyWeights,
}

/// Atomic resume state emitted when a micro-training shard is preempted mid-flight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroTrainCheckpoint {
    pub task_id: [u8; 32],
    pub shard_id: u16,
    pub completed_steps: u16,
    pub total_steps: u16,
    pub weights_f16_bits: Vec<u16>,
    pub last_loss_micro: u64,
    pub checkpoint_digest: [u8; 32],
}

impl MicroTrainCheckpoint {
    pub fn compute_digest(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(CHECKPOINT_DIGEST_DOMAIN);
        hasher.update(&self.task_id);
        hasher.update(&self.shard_id.to_le_bytes());
        hasher.update(&self.completed_steps.to_le_bytes());
        hasher.update(&self.total_steps.to_le_bytes());
        hasher.update(&self.last_loss_micro.to_le_bytes());
        let len = self.weights_f16_bits.len() as u32;
        hasher.update(&len.to_le_bytes());
        for &bits in &self.weights_f16_bits {
            hasher.update(&bits.to_le_bytes());
        }
        *hasher.finalize().as_bytes()
    }

    pub fn seal(mut self) -> Self {
        self.checkpoint_digest = self.compute_digest();
        self
    }

    pub fn verify_integrity(&self) -> Result<(), CheckpointError> {
        if self.weights_f16_bits.is_empty() {
            return Err(CheckpointError::EmptyWeights);
        }
        if self.completed_steps > self.total_steps || self.total_steps == 0 {
            return Err(CheckpointError::InvalidStepProgress {
                completed: self.completed_steps,
                total: self.total_steps,
            });
        }
        if self.compute_digest() != self.checkpoint_digest {
            return Err(CheckpointError::DigestMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_latency_guard_triggers_preempt_above_1_8x_after_warmup() {
        let guard = PreemptGuard::new();
        let mut latency_guard = SliceLatencyGuard::new();
        for _ in 0..SLICE_WARMUP_COUNT {
            assert_eq!(latency_guard.observe_slice(1_000, &guard), PreemptSignal::Run);
        }
        assert_eq!(latency_guard.observe_slice(1_750, &guard), PreemptSignal::Run);
        let ema = latency_guard.ema_slice_micros();
        assert_eq!(latency_guard.observe_slice((ema * 18 / 10) + 5, &guard), PreemptSignal::PreemptImmediate);
        assert!(guard.should_preempt());
    }

    #[test]
    fn micro_train_checkpoint_detects_tampering() {
        let mut ckpt = MicroTrainCheckpoint {
            task_id: [9u8; 32],
            shard_id: 2,
            completed_steps: 3,
            total_steps: 8,
            weights_f16_bits: vec![0x3c00, 0xbc00],
            last_loss_micro: 125_000,
            checkpoint_digest: [0u8; 32],
        }
        .seal();
        assert!(ckpt.verify_integrity().is_ok());
        ckpt.weights_f16_bits[0] ^= 1;
        assert_eq!(ckpt.verify_integrity(), Err(CheckpointError::DigestMismatch));
    }
}
