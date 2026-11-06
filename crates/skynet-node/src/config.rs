//! Configuration and command-line parsing for `skynet-node`.

use std::net::SocketAddr;
use std::path::PathBuf;
use skynet_cell::HardwareTier;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing argument value for: {0}")]
    MissingValue(String),
    #[error("invalid port: {0}")]
    InvalidPort(String),
    #[error("invalid hardware tier '{0}', must be TINY, STANDARD, or COLOSSAL")]
    InvalidTier(String),
    #[error("invalid seed address '{0}': {1}")]
    InvalidSeed(String, String),
    #[error("invalid owner pubkey hex '{0}': {1}")]
    InvalidOwnerPubkey(String, String),
    #[error("unknown command or flag: {0}")]
    UnknownFlag(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Run(NodeConfig),
    Status { data_dir: PathBuf },
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub port: u16,
    pub data_dir: PathBuf,
    pub name: String,
    pub tier: HardwareTier,
    pub seed_peers: Vec<SocketAddr>,
    pub owner_wallet: Option<[u8; 32]>,
    pub tick_interval_ms: u64,
    pub auto_flush_interval_sec: u64,
}

impl Default for NodeConfig {
    fn default() -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        let default_dir = home.join(".skynet").join("data");

        Self {
            port: 8888,
            data_dir: default_dir,
            name: "Skynet-Node".to_string(),
            tier: HardwareTier::StandardNode,
            seed_peers: Vec::new(),
            owner_wallet: None,
            tick_interval_ms: 1000,
            auto_flush_interval_sec: 30,
        }
    }
}

impl NodeConfig {
    pub fn parse(args: &[String]) -> Result<CliCommand, ConfigError> {
        if args.is_empty() {
            return Ok(CliCommand::Run(Self::default()));
        }
        if args[0] == "help" || args[0] == "--help" || args[0] == "-h" {
            return Ok(CliCommand::Help);
        }
        if args[0] == "status" {
            return Self::parse_status_cmd(&args[1..]);
        }
        let mut cfg = Self::default();
        let mut idx = if args[0] == "run" { 1 } else { 0 };
        while idx < args.len() {
            idx = Self::parse_next_flag(args, idx, &mut cfg)?;
        }
        Ok(CliCommand::Run(cfg))
    }

    fn parse_status_cmd(args: &[String]) -> Result<CliCommand, ConfigError> {
        let mut data_dir = Self::default().data_dir;
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--data-dir" {
                let val = args.get(i + 1).ok_or_else(|| {
                    ConfigError::MissingValue("--data-dir".into())
                })?;
                data_dir = PathBuf::from(val);
                i += 2;
            } else {
                return Err(ConfigError::UnknownFlag(args[i].clone()));
            }
        }
        Ok(CliCommand::Status { data_dir })
    }

    fn parse_next_flag(
        args: &[String],
        i: usize,
        cfg: &mut NodeConfig,
    ) -> Result<usize, ConfigError> {
        let flag = &args[i];
        let val = args.get(i + 1).ok_or_else(|| ConfigError::MissingValue(flag.clone()))?;
        match flag.as_str() {
            "--port" | "-p" => {
                cfg.port = val.parse().map_err(|_| ConfigError::InvalidPort(val.clone()))?;
            }
            "--data-dir" | "-d" => {
                cfg.data_dir = PathBuf::from(val);
            }
            "--name" | "-n" => {
                cfg.name = val.clone();
            }
            "--tier" | "-t" => {
                cfg.tier = Self::parse_tier(val)?;
            }
            "--seed" | "-s" => {
                let addr = val.parse().map_err(|e: std::net::AddrParseError| {
                    ConfigError::InvalidSeed(val.clone(), e.to_string())
                })?;
                cfg.seed_peers.push(addr);
            }
            "--owner" => {
                cfg.owner_wallet = Some(Self::parse_hex_pubkey(val)?);
            }
            _ => return Err(ConfigError::UnknownFlag(flag.clone())),
        }
        Ok(i + 2)
    }

    fn parse_tier(val: &str) -> Result<HardwareTier, ConfigError> {
        match val.to_uppercase().as_str() {
            "TINY" | "TINYPERIPHERAL" => Ok(HardwareTier::TinyPeripheral),
            "STANDARD" | "STANDARDNODE" => Ok(HardwareTier::StandardNode),
            "COLOSSAL" | "COLOSSALORGANISM" => Ok(HardwareTier::ColossalOrganism),
            _ => Err(ConfigError::InvalidTier(val.to_string())),
        }
    }

    fn parse_hex_pubkey(val: &str) -> Result<[u8; 32], ConfigError> {
        let trimmed = val.strip_prefix("0x").unwrap_or(val);
        if trimmed.len() != 64 {
            return Err(ConfigError::InvalidOwnerPubkey(
                val.to_string(),
                "must be 32 bytes (64 hex characters)".to_string(),
            ));
        }
        let mut key = [0u8; 32];
        for (i, chunk) in trimmed.as_bytes().chunks(2).enumerate() {
            let str_byte = std::str::from_utf8(chunk)
                .map_err(|e| ConfigError::InvalidOwnerPubkey(val.to_string(), e.to_string()))?;
            key[i] = u8::from_str_radix(str_byte, 16)
                .map_err(|e| ConfigError::InvalidOwnerPubkey(val.to_string(), e.to_string()))?;
        }
        Ok(key)
    }
}
