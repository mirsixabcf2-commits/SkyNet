//! `worker_cli`: Turnkey CLI configuration and flag parser for the `skynet-agent`
//! opportunistic edge worker daemon.

use std::net::SocketAddr;
use std::path::PathBuf;

use skynet_agent::{
    GovernorConfig, DEFAULT_HYSTERESIS_COOL_C, DEFAULT_IDLE_HOLD_SECS, DEFAULT_MAX_VRAM_PCT,
    DEFAULT_TEMP_LIMIT_C,
};
use skynet_ledger::{NanoSkn, YieldSweepConfig, DEFAULT_AUTO_SWEEP_SKN, NANO_PER_SKN};
use thiserror::Error;

use crate::cli::decode_hex_32;

const DEFAULT_OPERATING_RESERVE_SKN: u128 = 10;

#[derive(Debug, Error)]
pub enum WorkerCliError {
    #[error("missing value for flag: {0}")]
    MissingValue(String),
    #[error("invalid numeric value for {0}: {1}")]
    InvalidNumber(String, String),
    #[error("invalid socket address '{0}': {1}")]
    InvalidAddress(String, String),
    #[error("invalid --owner hex pubkey: {0}")]
    InvalidOwnerKey(String),
    #[error("invalid --schedule window '{0}', expected HH:MM-HH:MM")]
    InvalidSchedule(String),
    #[error("unknown worker CLI flag: {0}")]
    UnknownFlag(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkerCliCommand {
    Help,
    InitWorker(Box<WorkerCliConfig>),
    Run(Box<WorkerCliConfig>),
}

/// Complete configuration for an opportunistic or classic `skynet-agent` worker process.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkerCliConfig {
    pub opportunistic: bool,
    pub listen_port: u16,
    pub auto_bid: bool,
    pub owner_pubkey: Option<[u8; 32]>,
    pub seed_peers: Vec<SocketAddr>,
    pub genesis_path: Option<PathBuf>,
    pub schedule_window: Option<(u16, u16)>,
    pub auto_sweep_skn: u64,
    pub game_watchlist: Vec<String>,
    pub max_vram_pct: u8,
    pub idle_hold_secs: u64,
    pub battery_guard: bool,
    pub temp_limit_c: u8,
    pub tags: Vec<String>,
    pub min_bounty_rate: f64,
    pub wallet_path: PathBuf,
    pub data_dir: PathBuf,
}

impl Default for WorkerCliConfig {
    fn default() -> Self {
        Self {
            opportunistic: false,
            listen_port: 8889,
            auto_bid: false,
            owner_pubkey: None,
            seed_peers: Vec::new(),
            genesis_path: None,
            schedule_window: None,
            auto_sweep_skn: DEFAULT_AUTO_SWEEP_SKN as u64,
            game_watchlist: Vec::new(),
            max_vram_pct: DEFAULT_MAX_VRAM_PCT,
            idle_hold_secs: DEFAULT_IDLE_HOLD_SECS,
            battery_guard: true,
            temp_limit_c: DEFAULT_TEMP_LIMIT_C,
            tags: vec!["cpu".into(), "ai".into(), "etl".into()],
            min_bounty_rate: 1.0,
            wallet_path: PathBuf::from("wallet.json"),
            data_dir: PathBuf::from(".skynet_worker"),
        }
    }
}

impl WorkerCliConfig {
    pub fn parse(args: &[String]) -> Result<WorkerCliCommand, WorkerCliError> {
        if args.iter().any(|a| a == "--help" || a == "-h" || a == "help") {
            return Ok(WorkerCliCommand::Help);
        }
        let mut cfg = Self::default();
        let mut init_worker = false;
        let mut idx = 0usize;
        while idx < args.len() {
            if args[idx] == "--init-worker" {
                init_worker = true;
                idx += 1;
                continue;
            }
            idx = Self::parse_one_arg(args, idx, &mut cfg)?;
        }
        if init_worker {
            return Ok(WorkerCliCommand::InitWorker(Box::new(cfg)));
        }
        Ok(WorkerCliCommand::Run(Box::new(cfg)))
    }

    fn parse_one_arg(args: &[String], idx: usize, cfg: &mut Self) -> Result<usize, WorkerCliError> {
        match args[idx].as_str() {
            "--opportunistic" => {
                cfg.opportunistic = true;
                cfg.auto_bid = true;
                Ok(idx + 1)
            }
            "--auto-bid" => {
                cfg.auto_bid = true;
                Ok(idx + 1)
            }
            "--battery-guard" => {
                cfg.battery_guard = true;
                Ok(idx + 1)
            }
            "--no-battery-guard" => {
                cfg.battery_guard = false;
                Ok(idx + 1)
            }
            flag => Self::parse_valued_flag(args, idx, flag, cfg),
        }
    }

    fn parse_valued_flag(args: &[String], idx: usize, flag: &str, cfg: &mut Self) -> Result<usize, WorkerCliError> {
        let (key, val, step) = extract_key_value(args, idx, flag)?;
        match key {
            "--listen" | "-p" => cfg.listen_port = parse_num(key, val)?,
            "--owner" => cfg.owner_pubkey = Some(decode_hex_32(val).map_err(|e| WorkerCliError::InvalidOwnerKey(e.to_string()))?),
            "--seed" | "-s" => cfg.seed_peers.push(val.parse().map_err(|e: std::net::AddrParseError| WorkerCliError::InvalidAddress(val.into(), e.to_string()))?),
            "--genesis" => cfg.genesis_path = Some(PathBuf::from(val)),
            "--schedule" => cfg.schedule_window = Some(parse_schedule_window(val)?),
            "--auto-sweep-skn" => cfg.auto_sweep_skn = parse_num(key, val)?,
            "--game-watchlist" => cfg.game_watchlist = split_csv(val),
            "--max-vram-pct" => cfg.max_vram_pct = parse_num::<u8>(key, val)?.min(100),
            "--idle-hold-secs" => cfg.idle_hold_secs = parse_num(key, val)?,
            "--temp-limit" => cfg.temp_limit_c = parse_num(key, val)?,
            "--tags" => cfg.tags = split_csv(val),
            "--min-bounty-rate" => cfg.min_bounty_rate = parse_num(key, val)?,
            "--wallet-path" => cfg.wallet_path = PathBuf::from(val),
            "--data-dir" => cfg.data_dir = PathBuf::from(val),
            _ => return Err(WorkerCliError::UnknownFlag(key.to_string())),
        }
        Ok(idx + step)
    }

    pub fn to_governor_config(&self) -> GovernorConfig {
        GovernorConfig {
            temp_limit_c: self.temp_limit_c,
            hysteresis_cool_c: DEFAULT_HYSTERESIS_COOL_C,
            idle_hold_secs: self.idle_hold_secs,
            max_vram_pct: self.max_vram_pct,
            battery_guard: self.battery_guard,
            schedule_window: self.schedule_window,
        }
    }

    pub fn to_yield_sweep_config(&self) -> Option<YieldSweepConfig> {
        let human_owner = self.owner_pubkey?;
        Some(YieldSweepConfig {
            human_owner,
            operating_reserve: NanoSkn(DEFAULT_OPERATING_RESERVE_SKN * NANO_PER_SKN),
            auto_sweep_threshold: NanoSkn(u128::from(self.auto_sweep_skn) * NANO_PER_SKN),
        })
    }
}

fn extract_key_value<'a>(args: &'a [String], idx: usize, raw: &'a str) -> Result<(&'a str, &'a str, usize), WorkerCliError> {
    if let Some((k, v)) = raw.split_once('=') {
        return Ok((k, v, 1));
    }
    let val = args.get(idx + 1).map(String::as_str).ok_or_else(|| WorkerCliError::MissingValue(raw.into()))?;
    Ok((raw, val, 2))
}

fn parse_num<T: std::str::FromStr>(key: &str, val: &str) -> Result<T, WorkerCliError>
where
    T::Err: std::fmt::Display,
{
    val.parse::<T>().map_err(|e| WorkerCliError::InvalidNumber(key.into(), e.to_string()))
}

fn split_csv(val: &str) -> Vec<String> {
    val.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// Parses `HH:MM-HH:MM` into minutes-from-midnight `(start_min, end_min)`.
pub fn parse_schedule_window(spec: &str) -> Result<(u16, u16), WorkerCliError> {
    let (s, e) = spec.split_once('-').ok_or_else(|| WorkerCliError::InvalidSchedule(spec.into()))?;
    Ok((parse_hhmm(s, spec)?, parse_hhmm(e, spec)?))
}

fn parse_hhmm(part: &str, full: &str) -> Result<u16, WorkerCliError> {
    let (h_s, m_s) = part.trim().split_once(':').ok_or_else(|| WorkerCliError::InvalidSchedule(full.into()))?;
    let h: u16 = h_s.parse().map_err(|_| WorkerCliError::InvalidSchedule(full.into()))?;
    let m: u16 = m_s.parse().map_err(|_| WorkerCliError::InvalidSchedule(full.into()))?;
    if h > 23 || m > 59 {
        return Err(WorkerCliError::InvalidSchedule(full.into()));
    }
    Ok(h * 60 + m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turnkey_worker_cli_flags_and_schedule_parsing() {
        let owner_hex = "ab".repeat(32);
        let raw = ["--opportunistic", "--owner", &owner_hex, "--seed", "127.0.0.1:8888", "--schedule", "22:00-07:30", "--auto-sweep-skn", "80", "--game-watchlist", "starfield,rust", "--max-vram-pct", "60", "--idle-hold-secs", "45"]
            .into_iter().map(String::from).collect::<Vec<_>>();
        let WorkerCliCommand::Run(cfg) = WorkerCliConfig::parse(&raw).unwrap() else { panic!("expected Run") };
        assert!(cfg.opportunistic && cfg.auto_bid && cfg.battery_guard);
        assert_eq!((cfg.schedule_window, cfg.auto_sweep_skn, cfg.max_vram_pct, cfg.idle_hold_secs), (Some((1320, 450)), 80, 60, 45));
        assert!(cfg.to_yield_sweep_config().is_some());
    }
}
