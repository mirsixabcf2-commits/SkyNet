//! `skynet-node`: Production daemon service and persistent client suite for SkyNet nodes.

pub mod config;
pub mod identity;
pub mod preprocessor;
pub mod presets;
pub mod runtime;
pub mod storage;
pub mod storage_types;

pub use config::{CliCommand, NodeConfig};
pub use identity::NodeIdentity;
pub use preprocessor::{
    clean_csv, decrypt_envelope, detect_delimiter, encrypt_envelope, generate_ephemeral_key,
    redact_pii, run_preprocessing, sliding_window_chunks, standardize_e164_vietnam, to_candle_task,
    CleanCsvOutput, ImputeStrategy, PreprocessOptions, PreprocessResult,
};
pub use presets::{default_options_for_preset, find_preset, list_presets, TurnkeyPreset};
pub use runtime::{NodeRuntime, RuntimeError};
pub use storage::{StorageEngine, StorageError};
pub use storage_types::{
    FullNodeSnapshot, LedgerSnapshot, NodeMetadata, OrganismSnapshot, PeerTableSnapshot,
};
