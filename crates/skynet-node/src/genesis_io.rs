//! `genesis_io`: Atomic canonical JSON persistence and verification for `genesis.json`.

use std::fs;
use std::path::{Path, PathBuf};

use skynet_ledger::{
    compute_genesis_hash, validate_genesis_invariants, GenesisConfig, LedgerError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenesisIoError {
    #[error("genesis filesystem I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("genesis JSON codec error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("genesis supply/metadata invariant violation: {0}")]
    Ledger(#[from] LedgerError),
}

/// Loads `genesis.json` from `path`, enforces `21T SKN` (`85%` / `15%`) invariants,
/// and returns `(GenesisConfig, GENESIS_HASH)`.
pub fn load_genesis_file(path: &Path) -> Result<(GenesisConfig, [u8; 32]), GenesisIoError> {
    let raw = fs::read_to_string(path)?;
    let config: GenesisConfig = serde_json::from_str(&raw)?;
    validate_genesis_invariants(&config)?;
    let hash = compute_genesis_hash(&config);
    Ok((config, hash))
}

/// Validates `config`, writes pretty-printed `genesis.json` atomically via `.tmp -> rename`,
/// and returns the deterministic 32-byte `GENESIS_HASH`.
pub fn write_genesis_file(path: &Path, config: &GenesisConfig) -> Result<[u8; 32], GenesisIoError> {
    validate_genesis_invariants(config)?;
    let hash = compute_genesis_hash(config);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let payload = serde_json::to_string_pretty(config)?;
    let tmp_path = atomic_tmp_path(path);
    fs::write(&tmp_path, payload.as_bytes())?;
    fs::rename(&tmp_path, path)?;
    Ok(hash)
}

fn atomic_tmp_path(target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("genesis.json");
    let pid = std::process::id();
    target.with_file_name(format!(".{file_name}.{pid}.tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use skynet_ledger::default_mainnet_genesis;

    #[test]
    fn test_genesis_io_roundtrip_and_tamper_rejection() {
        let dir = std::env::temp_dir().join(format!("skynet_gen_io_{}", rand::random::<u64>()));
        let path = dir.join("genesis.json");
        let cfg = default_mainnet_genesis();

        let written_hash = write_genesis_file(&path, &cfg).expect("write genesis must succeed");
        let (loaded_cfg, loaded_hash) = load_genesis_file(&path).expect("load genesis must succeed");
        assert_eq!(written_hash, loaded_hash);
        assert_eq!(loaded_cfg, cfg);

        let mut tampered = loaded_cfg;
        tampered.initial_utxos[0].amount.0 += 1;
        fs::write(&path, serde_json::to_string_pretty(&tampered).unwrap()).unwrap();
        let err = load_genesis_file(&path).expect_err("1-nanoSKN tamper must fail");
        assert!(matches!(
            err,
            GenesisIoError::Ledger(LedgerError::GenesisSupplyMismatch { .. })
        ));

        let _ = fs::remove_dir_all(dir);
    }
}
