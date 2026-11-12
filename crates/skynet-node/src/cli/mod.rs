//! `cli`: Modular command-line argument utilities, error types, and subcommand handlers
//! for the `skynet` client binary.

pub mod compute_cmd;
pub mod genesis_airdrop_cmd;
pub mod wallet_opcode_cmd;

use std::fs;
use std::path::PathBuf;

use ed25519_dalek::SigningKey;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use skynet_cell::pope_benchmark::PopeError;
use skynet_ledger::hwid_airdrop::MarketingError;
use skynet_ledger::LedgerError;
use skynet_wire::WireError;
use thiserror::Error;

use crate::genesis_io::GenesisIoError;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("bincode error: {0}")]
    Bincode(#[from] Box<bincode::ErrorKind>),
    #[error("genesis file error: {0}")]
    GenesisIo(#[from] GenesisIoError),
    #[error("ledger error: {0}")]
    Ledger(#[from] LedgerError),
    #[error("marketing bootstrap error: {0}")]
    Marketing(#[from] MarketingError),
    #[error("PoPE attestation error: {0}")]
    Pope(#[from] PopeError),
    #[error("wire protocol error: {0}")]
    Wire(#[from] WireError),
    #[error("preprocessing error: {0}")]
    Preprocess(String),
    #[error("invalid hex string: {0}")]
    InvalidHex(String),
    #[error("missing required argument: {0}")]
    MissingArg(String),
    #[error("invalid argument value: {0}")]
    InvalidValue(String),
}

/// Prints the top-level `skynet` CLI help text.
pub fn print_cli_help() {
    println!("SkyNet Bio-Cellular AI User Client & AI Dispatcher CLI\n");
    println!("USAGE:");
    println!("  skynet <COMMAND> [SUBCOMMAND] [OPTIONS]\n");
    println!("COMMANDS:");
    println!("  wallet init [--path <path>]                   Initialize a new cryptographic Ed25519 wallet");
    println!("  wallet balance [--wallet <path>]              Check on-chain NanoSkn balance");
    println!("  genesis generate --output <path> [--network]  Generate deterministic 21T SKN genesis.json");
    println!("  genesis verify --path <path>                  Verify genesis.json invariants & GENESIS_HASH");
    println!("  airdrop claim [--wallet <path>]               Run PoPE benchmark & claim HWID AgentBond");
    println!("  opcode list                                   List standardized compute domains");
    println!("  opcode search <keyword>                       Search opcodes by name/keyword");
    println!("  opcode info <0xHEX>                           Inspect opcode hardware specs and tier");
    println!("  compute presets                               List turnkey data preprocessing presets");
    println!("  compute dispatch [OPTIONS]                    Preprocess and dispatch compute task over UDP");
    println!("  compute train --input <path> [OPTIONS]        Stream Fountain-coded micro-training shards");
}

/// Extracts `--flag <val>` or `--flag=<val>` from `args`.
pub fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    let eq_prefix = format!("{flag}=");
    for (i, a) in args.iter().enumerate() {
        if a == flag && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
        if let Some(val) = a.strip_prefix(&eq_prefix) {
            return Some(val.to_string());
        }
    }
    None
}

/// Checks whether boolean `flag` is present in `args`.
pub fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

/// Formats arbitrary bytes as lowercase hexadecimal.
pub fn encode_hex<T: AsRef<[u8]>>(data: T) -> String {
    data.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

/// Decodes a 64-character hex string (with optional `0x` prefix) into `[u8; 32]`.
pub fn decode_hex_32(hex_str: &str) -> Result<[u8; 32], CliError> {
    let trimmed = hex_str.trim().strip_prefix("0x").unwrap_or(hex_str.trim());
    if trimmed.len() != 64 {
        return Err(CliError::InvalidHex(format!("expected 64 hex chars, got {}", trimmed.len())));
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        let slice = trimmed
            .get(i * 2..i * 2 + 2)
            .ok_or_else(|| CliError::InvalidHex("non-ASCII hex character".into()))?;
        *byte = u8::from_str_radix(slice, 16).map_err(|e| CliError::InvalidHex(e.to_string()))?;
    }
    Ok(out)
}

/// Reads the `public_key` hex field from a wallet JSON file (or returns zero hex if absent).
pub fn read_wallet_pubkey_hex(path: &str) -> Result<String, CliError> {
    let p = PathBuf::from(path);
    if !p.exists() {
        return Ok("0".repeat(64));
    }
    let val: serde_json::Value = serde_json::from_str(&fs::read_to_string(p)?)?;
    Ok(val.get("public_key").and_then(|v| v.as_str()).unwrap_or(&"0".repeat(64)).to_string())
}

/// Loads `(SigningKey, pubkey)` from `wallet.json`, or derives an ephemeral key when absent.
pub fn load_or_ephemeral_wallet_key(path: &str) -> Result<(SigningKey, [u8; 32]), CliError> {
    let p = PathBuf::from(path);
    if !p.exists() {
        let mut rng = ChaCha8Rng::from_entropy();
        let sk = SigningKey::generate(&mut rng);
        let pk = sk.verifying_key().to_bytes();
        return Ok((sk, pk));
    }
    let val: serde_json::Value = serde_json::from_str(&fs::read_to_string(p)?)?;
    let secret_hex = val
        .get("secret_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::MissingArg("wallet JSON missing secret_key".into()))?;
    let sk = SigningKey::from_bytes(&decode_hex_32(secret_hex)?);
    let pk = sk.verifying_key().to_bytes();
    Ok((sk, pk))
}
