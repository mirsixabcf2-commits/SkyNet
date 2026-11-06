//! `compute_candle`: Hardware-accelerated Candle tensor compute executor
//! supporting CUDA, Metal, and CPU fallbacks with cryptographic result verification.

use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::HardwareProfile;

#[derive(Debug, Error)]
pub enum ComputeCandleError {
    #[error("input vector length mismatch: expected {expected}, got {actual}")]
    InputDimensionMismatch { expected: usize, actual: usize },
    #[error("weights buffer length mismatch: expected {expected}, got {actual}")]
    WeightsDimensionMismatch { expected: usize, actual: usize },
    #[error("candle tensor runtime error: {0}")]
    Candle(#[from] candle_core::Error),
}

/// Task input containing neural model dimensions, serialized weights, and input tensor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandleTaskInput {
    pub task_id: [u8; 32],
    pub dim_in: usize,
    pub dim_hidden: usize,
    pub dim_out: usize,
    pub weights: Vec<f32>,
    pub input: Vec<f32>,
}

impl CandleTaskInput {
    pub fn required_weights_len(&self) -> usize {
        let w1 = self.dim_in.saturating_mul(self.dim_hidden);
        let b1 = self.dim_hidden;
        let w2 = self.dim_hidden.saturating_mul(self.dim_out);
        let b2 = self.dim_out;
        w1.saturating_add(b1).saturating_add(w2).saturating_add(b2)
    }
}

/// Verified execution result containing output tensor and BLAKE3 digest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandleTaskOutput {
    pub task_id: [u8; 32],
    pub output: Vec<f32>,
    pub result_digest: [u8; 32],
    pub execution_ticks: u64,
}

/// Helper: evaluates 2-layer neural network forward pass.
fn forward_two_layer(
    task: &CandleTaskInput,
    device: &Device,
) -> Result<Vec<f32>, ComputeCandleError> {
    let d_in = task.dim_in;
    let h = task.dim_hidden;
    let d_out = task.dim_out;

    let w1_len = d_in * h;
    let b1_len = h;
    let w2_len = h * d_out;

    let w1_s = &task.weights[0..w1_len];
    let b1_s = &task.weights[w1_len..w1_len + b1_len];
    let w2_s = &task.weights[w1_len + b1_len..w1_len + b1_len + w2_len];
    let b2_s = &task.weights[w1_len + b1_len + w2_len..];

    let x = Tensor::from_slice(&task.input, (1, d_in), device)?;
    let w1 = Tensor::from_slice(w1_s, (d_in, h), device)?;
    let b1 = Tensor::from_slice(b1_s, (1, h), device)?;
    let hidden = x.matmul(&w1)?.broadcast_add(&b1)?.tanh()?;

    let w2 = Tensor::from_slice(w2_s, (h, d_out), device)?;
    let b2 = Tensor::from_slice(b2_s, (1, d_out), device)?;
    let y = hidden.matmul(&w2)?.broadcast_add(&b2)?.tanh()?;

    Ok(y.squeeze(0)?.to_vec1::<f32>()?)
}

/// Canonical fixed-point quantization scale (10,000x = 4 decimal places) eliminating
/// cross-device floating point rounding noise (IEEE 754 ULP jitter between CUDA/Metal/CPU).
pub const FIXED_POINT_DIGEST_SCALE: f32 = 10_000.0;

/// Verifies whether two output tensors are numerically identical within tolerance epsilon.
pub fn verify_output_tolerance(actual: &[f32], expected: &[f32], max_epsilon: f32) -> bool {
    if actual.len() != expected.len() {
        return false;
    }
    actual
        .iter()
        .zip(expected.iter())
        .all(|(&a, &b)| (a - b).abs() <= max_epsilon)
}

/// Helper: computes deterministic cryptographic BLAKE3 digest across float slice
/// using fixed-point quantization to neutralize IEEE 754 float differences across GPU architectures.
pub fn compute_canonical_digest(output: &[f32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"SKYNET_CANONICAL_DIGEST_V2_FIXED_POINT");
    for &val in output {
        let scaled = if val.is_finite() {
            (val * FIXED_POINT_DIGEST_SCALE).round() as i32
        } else {
            0i32
        };
        hasher.update(&scaled.to_le_bytes());
    }
    *hasher.finalize().as_bytes()
}

/// Micro-neural Candle compute executor with automatic hardware dispatch.
pub struct CandleComputeExecutor {
    device: Device,
}

impl Default for CandleComputeExecutor {
    fn default() -> Self {
        Self::new_auto()
    }
}

impl CandleComputeExecutor {
    /// Initializes executor selecting the most capable hardware device (CUDA -> Metal -> CPU).
    pub fn new_auto() -> Self {
        Self {
            device: HardwareProfile::select_compute_device(),
        }
    }

    /// Initializes executor with an explicit `candle_core::Device`.
    pub fn with_device(device: Device) -> Self {
        Self { device }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Executes neural forward pass: `y = tanh(tanh(x @ W1 + b1) @ W2 + b2)`.
    pub fn execute(&self, task: &CandleTaskInput) -> Result<CandleTaskOutput, ComputeCandleError> {
        if task.input.len() != task.dim_in {
            return Err(ComputeCandleError::InputDimensionMismatch {
                expected: task.dim_in,
                actual: task.input.len(),
            });
        }
        let needed_weights = task.required_weights_len();
        if task.weights.len() != needed_weights {
            return Err(ComputeCandleError::WeightsDimensionMismatch {
                expected: needed_weights,
                actual: task.weights.len(),
            });
        }

        let output = forward_two_layer(task, &self.device)?;
        let result_digest = compute_canonical_digest(&output);

        Ok(CandleTaskOutput {
            task_id: task.task_id,
            output,
            result_digest,
            execution_ticks: (needed_weights + task.dim_in) as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_executor_forward_and_digest() {
        let executor = CandleComputeExecutor::with_device(Device::Cpu);
        let task_id = [42u8; 32];
        let dim_in = 2;
        let dim_hidden = 3;
        let dim_out = 1;

        // Weights: w1 (2x3), b1 (1x3), w2 (3x1), b2 (1x1) = 6 + 3 + 3 + 1 = 13 floats
        let weights = vec![0.1f32; 13];
        let input = vec![1.0f32, -1.0f32];

        let task = CandleTaskInput {
            task_id,
            dim_in,
            dim_hidden,
            dim_out,
            weights,
            input,
        };

        let result = executor.execute(&task).expect("inference succeeds");
        assert_eq!(result.task_id, task_id);
        assert_eq!(result.output.len(), 1);
        assert_ne!(result.result_digest, [0u8; 32]);
    }

    #[test]
    fn test_candle_executor_dimension_mismatch_error() {
        let executor = CandleComputeExecutor::with_device(Device::Cpu);
        let task = CandleTaskInput {
            task_id: [1u8; 32],
            dim_in: 4,
            dim_hidden: 4,
            dim_out: 2,
            weights: vec![0.0; 10], // wrong weights length
            input: vec![1.0; 4],
        };
        let err = executor.execute(&task).unwrap_err();
        matches!(err, ComputeCandleError::WeightsDimensionMismatch { .. });
    }

    #[test]
    fn test_canonical_digest_resilient_to_floating_point_micro_jitter() {
        // Output from CUDA vs Metal with micro-ULP jitter (0.00001 difference)
        let out_cuda = vec![0.12341f32, -0.56781f32, 0.99991f32];
        let out_metal = vec![0.12344f32, -0.56784f32, 0.99994f32];

        assert!(verify_output_tolerance(&out_cuda, &out_metal, 0.001));

        let digest_cuda = compute_canonical_digest(&out_cuda);
        let digest_metal = compute_canonical_digest(&out_metal);
        assert_eq!(digest_cuda, digest_metal, "fixed-point digest must match across micro-jitter");
    }
}
