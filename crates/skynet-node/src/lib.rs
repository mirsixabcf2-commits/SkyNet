//! `skynet-node`: Production daemon service, rootless OS telemetry governor,
//! Genesis/P2P socket driver, and turnkey CLI suite for SkyNet nodes.

pub mod cli;
pub mod config;
pub mod genesis_io;
pub mod governor;
pub mod identity;
pub mod preprocessor;
pub mod presets;
pub mod runtime;
pub mod runtime_p2p;
pub mod storage;
pub mod storage_types;
pub mod worker_cli;

pub use config::{CliCommand, ConfigError, NodeConfig};
pub use genesis_io::{load_genesis_file, write_genesis_file, GenesisIoError};
pub use governor::{
    CheckpointStore, CheckpointStoreError, GovernorError, LinuxTelemetryProbe,
    MacOsTelemetryProbe, OsTelemetryGovernor, SimulatedTelemetryProbe, WindowsTelemetryProbe,
    WorkerStudioStateSnapshot, DEFAULT_GAME_WATCHLIST, WORKER_STUDIO_STATE_FILENAME,
};
pub use identity::NodeIdentity;
pub use preprocessor::{
    clean_csv, decrypt_envelope, detect_delimiter, encrypt_envelope, generate_ephemeral_key,
    redact_pii, run_preprocessing, sliding_window_chunks, standardize_e164_vietnam, to_candle_task,
    CleanCsvOutput, ImputeStrategy, PreprocessOptions, PreprocessResult,
};
pub use presets::{default_options_for_preset, find_preset, list_presets, TurnkeyPreset};
pub use runtime::{NodeRuntime, RuntimeError};
pub use runtime_p2p::{bootstrap_node_ledger, RuntimeP2pDriver, LAN_BEACON_INTERVAL_MS};
pub use storage::{StorageEngine, StorageError};
pub use storage_types::{
    FullNodeSnapshot, LedgerSnapshot, NodeMetadata, OrganismSnapshot, PeerTableSnapshot,
};
pub use worker_cli::{
    parse_schedule_window, WorkerCliCommand, WorkerCliConfig, WorkerCliError,
};
