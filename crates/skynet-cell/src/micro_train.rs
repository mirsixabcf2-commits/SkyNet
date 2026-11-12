//! Opportunistic multi-step MLP micro-training on `candle-core` with `< 15ms` slice
//! preemption, atomic `MicroTrainCheckpoint` resume, and `f16` weight-delta emission.

use std::time::{Duration, Instant};

use candle_core::backprop::GradStore;
use candle_core::{Device, Tensor, Var};

use crate::compute_candle::compute_canonical_digest;
use crate::gradient_codec::{clip_l2_norm, decode_dense_f16, encode_dense_f16_checked};
use crate::hardware::HardwareProfile;
use crate::preempt_guard::{MicroTrainCheckpoint, PreemptGuard, PreemptSignal, SliceLatencyGuard};

pub use crate::micro_train_types::{
    MicroTrainBatch, MicroTrainConfig, MicroTrainError, MicroTrainOutcome, MicroTrainResult,
};

const MICRO_SCALE: f32 = 1_000_000.0;
const MILLI_SCALE: f32 = 1_000.0;
const MAX_THROTTLE_SLEEP_MICROS: u64 = 5_000;

struct TrainSession<'a> {
    cfg: &'a MicroTrainConfig,
    batch: &'a MicroTrainBatch,
    base_w: Vec<f32>,
    vars: [Var; 4],
    x: Tensor,
    y: Tensor,
    initial_loss: f32,
    start_step: u16,
    last_loss: f32,
}

impl<'a> TrainSession<'a> {
    fn new(
        cfg: &'a MicroTrainConfig,
        batch: &'a MicroTrainBatch,
        resume: Option<&MicroTrainCheckpoint>,
    ) -> Result<Self, MicroTrainError> {
        cfg.validate()?;
        let (base_w, active_w, start_step, saved_loss) = prepare_weights(cfg, batch, resume)?;
        let dev = HardwareProfile::select_compute_device();
        let (x, y) = build_batch_tensors(cfg, batch, &dev)?;
        let base_vars = vars_from_flat(&base_w, cfg, &dev)?;
        let initial_loss = eval_mse(&base_vars, &x, &y)?.to_scalar::<f32>()?;
        let vars = vars_from_flat(&active_w, cfg, &dev)?;
        let last_loss = if start_step == 0 { initial_loss } else { saved_loss };
        Ok(Self { cfg, batch, base_w, vars, x, y, initial_loss, start_step, last_loss })
    }

    fn sgd_step(&mut self) -> Result<(), MicroTrainError> {
        let loss = eval_mse(&self.vars, &self.x, &self.y)?;
        let loss_val = loss.to_scalar::<f32>()?;
        if !loss_val.is_finite() {
            return Err(MicroTrainError::NonFiniteValue);
        }
        let grads = loss.backward()?;
        let grad_norm = grad_norm_sq(&grads, &self.vars)?.sqrt();
        let max_norm = (self.cfg.max_grad_norm_milli as f32) / MILLI_SCALE;
        let lr = (self.cfg.learning_rate_micro as f32) / MICRO_SCALE;
        let eff_lr = if grad_norm > max_norm && grad_norm > 0.0 { lr * (max_norm / grad_norm) } else { lr };
        for var in &self.vars {
            if let Some(g) = grads.get(var) {
                var.set(&var.as_tensor().sub(&(g * (eff_lr as f64))?)?)?;
            }
        }
        self.last_loss = loss_val;
        Ok(())
    }

    fn flat_weights(&self) -> Result<Vec<f32>, MicroTrainError> {
        let mut flat = Vec::with_capacity(self.base_w.len());
        for v in &self.vars {
            flat.extend(v.as_tensor().flatten_all()?.to_vec1::<f32>()?);
        }
        Ok(flat)
    }

    fn preempt_at(&self, completed_steps: u16) -> Result<MicroTrainOutcome, MicroTrainError> {
        let weights_f16_bits = encode_dense_f16_checked(&self.flat_weights()?)?;
        let ckpt = MicroTrainCheckpoint {
            task_id: self.batch.task_id,
            shard_id: self.batch.shard_id,
            completed_steps,
            total_steps: self.cfg.inner_steps,
            weights_f16_bits,
            last_loss_micro: (self.last_loss.max(0.0) * MICRO_SCALE).round() as u64,
            checkpoint_digest: [0u8; 32],
        }
        .seal();
        Ok(MicroTrainOutcome::Preempted(ckpt))
    }

    fn finalize(&self) -> Result<MicroTrainOutcome, MicroTrainError> {
        let final_loss = eval_mse(&self.vars, &self.x, &self.y)?.to_scalar::<f32>()?;
        if !self.initial_loss.is_finite() || !final_loss.is_finite() {
            return Err(MicroTrainError::NonFiniteValue);
        }
        let updated = self.flat_weights()?;
        let updated_weights_f16 = encode_dense_f16_checked(&updated)?;
        let mut raw_delta: Vec<f32> = updated.iter().zip(&self.base_w).map(|(&u, &b)| u - b).collect();
        clip_l2_norm(&mut raw_delta, (self.cfg.max_grad_norm_milli as f32) / MILLI_SCALE)?;
        let delta_f16 = encode_dense_f16_checked(&raw_delta)?;
        let canonical_digest = compute_canonical_digest(&decode_dense_f16(&delta_f16));
        Ok(MicroTrainOutcome::Completed(MicroTrainResult {
            task_id: self.batch.task_id,
            shard_id: self.batch.shard_id,
            completed_steps: self.cfg.inner_steps,
            delta_f16,
            updated_weights_f16,
            initial_loss: self.initial_loss,
            final_loss,
            canonical_digest,
        }))
    }
}

/// Executes `cfg.inner_steps` of Candle autograd MSE micro-training with `< 15ms` slice preemption.
pub fn execute_micro_train_slices(
    cfg: &MicroTrainConfig,
    batch: &MicroTrainBatch,
    resume: Option<&MicroTrainCheckpoint>,
    guard: &PreemptGuard,
) -> Result<MicroTrainOutcome, MicroTrainError> {
    let mut session = TrainSession::new(cfg, batch, resume)?;
    let mut latency_guard = SliceLatencyGuard::new();
    for step in session.start_step..cfg.inner_steps {
        if guard.should_preempt() {
            return session.preempt_at(step);
        }
        let t0 = Instant::now();
        session.sgd_step()?;
        let elapsed_us = t0.elapsed().as_micros() as u64;
        let sig = latency_guard.observe_slice(elapsed_us, guard);
        if sig == PreemptSignal::PreemptImmediate && step + 1 < cfg.inner_steps {
            return session.preempt_at(step + 1);
        }
        if sig == PreemptSignal::Throttle50 && step + 1 < cfg.inner_steps {
            std::thread::sleep(Duration::from_micros(elapsed_us.min(MAX_THROTTLE_SLEEP_MICROS)));
        }
    }
    session.finalize()
}

fn vars_from_flat(w: &[f32], cfg: &MicroTrainConfig, dev: &Device) -> Result<[Var; 4], MicroTrainError> {
    let (d_in, h, d_out) = (cfg.dim_in, cfg.dim_hidden, cfg.dim_out);
    let (i1, i2, i3) = (d_in * h, d_in * h + h, d_in * h + h + h * d_out);
    Ok([
        Var::from_tensor(&Tensor::from_slice(&w[..i1], (d_in, h), dev)?)?,
        Var::from_tensor(&Tensor::from_slice(&w[i1..i2], (1, h), dev)?)?,
        Var::from_tensor(&Tensor::from_slice(&w[i2..i3], (h, d_out), dev)?)?,
        Var::from_tensor(&Tensor::from_slice(&w[i3..], (1, d_out), dev)?)?,
    ])
}

fn eval_mse(vars: &[Var; 4], x: &Tensor, y: &Tensor) -> Result<Tensor, MicroTrainError> {
    let h = x.matmul(vars[0].as_tensor())?.broadcast_add(vars[1].as_tensor())?.tanh()?;
    let pred = h.matmul(vars[2].as_tensor())?.broadcast_add(vars[3].as_tensor())?;
    Ok((pred - y)?.sqr()?.mean_all()?)
}

fn grad_norm_sq(grads: &GradStore, vars: &[Var; 4]) -> Result<f32, MicroTrainError> {
    let mut sum = 0.0f32;
    for v in vars {
        if let Some(g) = grads.get(v) {
            sum += g.sqr()?.sum_all()?.to_scalar::<f32>()?;
        }
    }
    Ok(sum)
}

fn prepare_weights(
    cfg: &MicroTrainConfig,
    batch: &MicroTrainBatch,
    resume: Option<&MicroTrainCheckpoint>,
) -> Result<(Vec<f32>, Vec<f32>, u16, f32), MicroTrainError> {
    let expected = cfg.expected_weights_len();
    if batch.base_weights_f16.len() != expected {
        return Err(MicroTrainError::WeightsLengthMismatch { expected, actual: batch.base_weights_f16.len() });
    }
    let base_w = decode_dense_f16(&batch.base_weights_f16);
    if base_w.iter().any(|v| !v.is_finite()) {
        return Err(MicroTrainError::NonFiniteValue);
    }
    let Some(ckpt) = resume else { return Ok((base_w.clone(), base_w, 0, 0.0)); };
    ckpt.verify_integrity()?;
    if ckpt.task_id != batch.task_id || ckpt.shard_id != batch.shard_id || ckpt.total_steps != cfg.inner_steps {
        return Err(MicroTrainError::CheckpointTaskMismatch);
    }
    if ckpt.weights_f16_bits.len() != expected {
        return Err(MicroTrainError::WeightsLengthMismatch { expected, actual: ckpt.weights_f16_bits.len() });
    }
    Ok((base_w, decode_dense_f16(&ckpt.weights_f16_bits), ckpt.completed_steps, (ckpt.last_loss_micro as f32) / MICRO_SCALE))
}

fn build_batch_tensors(cfg: &MicroTrainConfig, batch: &MicroTrainBatch, dev: &Device) -> Result<(Tensor, Tensor), MicroTrainError> {
    let n = batch.samples_x.len() / cfg.dim_in;
    if n == 0 || n * cfg.dim_in != batch.samples_x.len() || batch.targets_y.len() != n * cfg.dim_out {
        return Err(MicroTrainError::BatchShapeMismatch { samples_len: batch.samples_x.len(), targets_len: batch.targets_y.len() });
    }
    if batch.samples_x.iter().any(|v| !v.is_finite()) || batch.targets_y.iter().any(|v| !v.is_finite()) {
        return Err(MicroTrainError::NonFiniteValue);
    }
    Ok((Tensor::from_slice(&batch.samples_x, (n, cfg.dim_in), dev)?, Tensor::from_slice(&batch.targets_y, (n, cfg.dim_out), dev)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gradient_codec::encode_dense_f16;

    #[test]
    fn micro_train_decreases_loss_and_resumes_from_preempt_checkpoint() {
        let cfg = MicroTrainConfig { dim_in: 2, dim_hidden: 4, dim_out: 1, inner_steps: 6, ..Default::default() };
        let batch = MicroTrainBatch {
            task_id: [3u8; 32],
            shard_id: 1,
            base_weights_f16: encode_dense_f16(&vec![0.1f32; cfg.expected_weights_len()]),
            samples_x: vec![1.0, -0.5, -0.8, 0.4, 0.3, 0.9],
            targets_y: vec![0.7, -0.6, 0.5],
        };
        let paused_guard = PreemptGuard::with_signal(PreemptSignal::PreemptImmediate);
        let MicroTrainOutcome::Preempted(ckpt) = execute_micro_train_slices(&cfg, &batch, None, &paused_guard).expect("preempt") else {
            panic!("expected immediate preemption");
        };
        assert_eq!(ckpt.completed_steps, 0);
        let run_guard = PreemptGuard::new();
        let MicroTrainOutcome::Completed(res) = execute_micro_train_slices(&cfg, &batch, Some(&ckpt), &run_guard).expect("resume") else {
            panic!("expected completed outcome");
        };
        assert!(res.final_loss < res.initial_loss);
        assert_eq!(res.canonical_digest, compute_canonical_digest(&res.decode_delta_f32()));
    }
}
