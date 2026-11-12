//! `skynet-cell`: Ultra-mini Liquid Neural AI cells powered by `candle-core`,
//! supporting Stem Cell differentiation, `1 -> 2` Mitosis, Neuro-evolution, Apoptosis,
//! opportunistic `< 100ms` preemptible micro-training, PoUW Canary spot-checks,
//! and Byzantine-resilient federated aggregation.

pub mod canary_pouw;
pub mod compute_candle;
pub mod federated_agg;
pub mod gradient_codec;
pub mod hardware;
pub mod micro_cell;
pub mod micro_train;
pub mod micro_train_types;
pub mod pope_benchmark;
pub mod preempt_guard;

#[cfg(test)]
mod cell_tests;

pub use canary_pouw::{
    generate_canary_batch, verify_canary_spot_check, CanaryConfig, CanaryVerdict, FraudReason,
    CANARY_XOF_DOMAIN, DEFAULT_CANARY_RATE_BPS, DEFAULT_MAX_GRAD_NORM_MILLI,
    DEFAULT_MIN_COSINE_MILLI,
};
pub use compute_candle::{
    compute_canonical_digest, verify_output_tolerance, CandleComputeExecutor, CandleTaskInput,
    CandleTaskOutput, ComputeCandleError, FIXED_POINT_DIGEST_SCALE,
};
pub use federated_agg::{
    AggregatedDelta, AggregationError, TrimmedMeanAggregator, WeightedDeltaSubmission,
    DEFAULT_AGG_MAX_NORM_MILLI, DEFAULT_TRIM_RATIO_BPS, MIN_SAMPLES_FOR_TRIM,
};
pub use gradient_codec::{
    clip_l2_norm, cosine_similarity, decode_dense_f16, decode_sparse_topk, encode_dense_f16,
    encode_dense_f16_checked, encode_sparse_topk, l2_norm, GradientCodecError, SparseTopKEntry,
};
pub use hardware::{CellError, HardwareProfile, HardwareTier};
pub use micro_cell::{MicroCell, TissueBatch};
pub use micro_train::execute_micro_train_slices;
pub use micro_train_types::{
    MicroTrainBatch, MicroTrainConfig, MicroTrainError, MicroTrainOutcome, MicroTrainResult,
};
pub use pope_benchmark::{
    compute_pope_challenge, verify_pope_proof, verify_pope_proof_with_latency,
    PopeChallengeParams, PopeError, PopeProof,
};
pub use preempt_guard::{
    CheckpointError, MicroTrainCheckpoint, PreemptGuard, PreemptSignal, SliceLatencyGuard,
    CHECKPOINT_DIGEST_DOMAIN, CONTENTION_SPIKE_DEN, CONTENTION_SPIKE_NUM, SLICE_WARMUP_COUNT,
};
