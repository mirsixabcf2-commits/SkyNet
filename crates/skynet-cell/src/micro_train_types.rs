//! Data contracts, configuration, outcomes, and error types for `micro_train`.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::gradient_codec::{decode_dense_f16, GradientCodecError};
use crate::preempt_guard::{CheckpointError, MicroTrainCheckpoint};

#[derive(Debug, Error)]
pub enum MicroTrainError {
    #[error("invalid micro-train configuration: {0}")]
    InvalidConfig(&'static str),
    #[error("weights length mismatch: expected {expected}, got {actual}")]
    WeightsLengthMismatch { expected: usize, actual: usize },
    #[error("batch sample/target dimension mismatch (samples_x={samples_len}, targets_y={targets_len})")]
    BatchShapeMismatch { samples_len: usize, targets_len: usize },
    #[error("non-finite value detected in training inputs or loss")]
    NonFiniteValue,
    #[error("checkpoint error: {0}")]
    Checkpoint(#[from] CheckpointError),
    #[error("checkpoint does not match batch task_id, shard_id, or total_steps")]
    CheckpointTaskMismatch,
    #[error("gradient codec error: {0}")]
    GradientCodec(#[from] GradientCodecError),
    #[error("candle runtime error: {0}")]
    Candle(#[from] candle_core::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroTrainConfig {
    pub dim_in: usize,
    pub dim_hidden: usize,
    pub dim_out: usize,
    pub inner_steps: u16,
    pub learning_rate_micro: u32,
    pub max_grad_norm_milli: u32,
}

impl Default for MicroTrainConfig {
    fn default() -> Self {
        Self {
            dim_in: 8,
            dim_hidden: 16,
            dim_out: 4,
            inner_steps: 8,
            learning_rate_micro: 50_000,
            max_grad_norm_milli: 5_000,
        }
    }
}

impl MicroTrainConfig {
    pub fn expected_weights_len(&self) -> usize {
        (self.dim_in * self.dim_hidden + self.dim_hidden)
            + (self.dim_hidden * self.dim_out + self.dim_out)
    }

    pub fn validate(&self) -> Result<(), MicroTrainError> {
        if self.dim_in == 0 || self.dim_hidden == 0 || self.dim_out == 0 {
            return Err(MicroTrainError::InvalidConfig("dimensions must be > 0"));
        }
        if self.inner_steps == 0 || self.learning_rate_micro == 0 || self.max_grad_norm_milli == 0 {
            return Err(MicroTrainError::InvalidConfig("steps, lr, and max_grad_norm must be > 0"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicroTrainBatch {
    pub task_id: [u8; 32],
    pub shard_id: u16,
    pub base_weights_f16: Vec<u16>,
    pub samples_x: Vec<f32>,
    pub targets_y: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MicroTrainResult {
    pub task_id: [u8; 32],
    pub shard_id: u16,
    pub completed_steps: u16,
    pub delta_f16: Vec<u16>,
    pub updated_weights_f16: Vec<u16>,
    pub initial_loss: f32,
    pub final_loss: f32,
    pub canonical_digest: [u8; 32],
}

impl MicroTrainResult {
    pub fn decode_delta_f32(&self) -> Vec<f32> {
        decode_dense_f16(&self.delta_f16)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MicroTrainOutcome {
    Completed(MicroTrainResult),
    Preempted(MicroTrainCheckpoint),
}
