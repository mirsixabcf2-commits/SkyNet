//! `wallet_opcode_cmd`: Cryptographic wallet management and standardized opcode catalog CLI.

use std::fs;
use std::path::Path;

use ed25519_dalek::SigningKey;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use skynet_ledger::NanoSkn;
use skynet_wire::opcode_registry::{lookup_opcode_descriptor, DOMAIN_CATEGORIES};

use crate::cli::{decode_hex_32, encode_hex, parse_flag, read_wallet_pubkey_hex, CliError};
use crate::storage::StorageEngine;

const NANO_PER_SKN_F64: f64 = 1_000_000_000.0;

/// Executes `skynet wallet <init|balance>`.
pub fn handle_wallet(args: &[String]) -> Result<(), CliError> {
    let sub = args.first().map(String::as_str).unwrap_or("help");
    let path = parse_flag(args, "--path")
        .or_else(|| parse_flag(args, "--wallet"))
        .unwrap_or_else(|| "wallet.json".to_string());

    match sub {
        "init" => init_wallet(&path),
        "balance" => print_wallet_balance(&path, args),
        _ => {
            eprintln!("Usage: skynet wallet [init|balance]");
            Ok(())
        }
    }
}

fn init_wallet(path: &str) -> Result<(), CliError> {
    let mut rng = ChaCha8Rng::from_entropy();
    let signing_key = SigningKey::generate(&mut rng);
    let pubkey_hex = encode_hex(signing_key.verifying_key().to_bytes());
    let secret_hex = encode_hex(signing_key.to_bytes());
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::json!({
        "public_key": pubkey_hex,
        "secret_key": secret_hex,
    });
    fs::write(path, serde_json::to_string_pretty(&json)?)?;
    println!("[+] Initialized wallet at: {path}");
    println!("    Public Address: 0x{pubkey_hex}");
    Ok(())
}

fn print_wallet_balance(path: &str, args: &[String]) -> Result<(), CliError> {
    let pubkey_hex = read_wallet_pubkey_hex(path)?;
    let data_dir = parse_flag(args, "--data-dir").unwrap_or_else(|| "./.skynet_data".to_string());
    let storage = StorageEngine::new(Path::new(&data_dir))
        .or_else(|_| StorageEngine::new(Path::new("~/.skynet/data")))
        .map_err(|e| CliError::InvalidValue(e.to_string()))?;
    let pubkey = decode_hex_32(&pubkey_hex)?;
    let balance = storage
        .load_ledger()
        .map_err(|e| CliError::InvalidValue(e.to_string()))?
        .map(|l| l.balance_of(&pubkey))
        .unwrap_or(NanoSkn::ZERO);
    println!("Wallet Balance for 0x{pubkey_hex}:");
    println!(
        "  On-Chain Balance: {} nanoSKN ({:.4} SKN)",
        balance.0,
        balance.0 as f64 / NANO_PER_SKN_F64
    );
    Ok(())
}

/// Executes `skynet opcode <list|search|info>`.
pub fn handle_opcode(args: &[String]) -> Result<(), CliError> {
    let sub = args.first().map(String::as_str).unwrap_or("list");
    match sub {
        "list" => print_opcode_domains(),
        "search" => search_opcode_domains(args.get(1).map(String::as_str).unwrap_or("")),
        "info" => print_opcode_info(args.get(1).map(String::as_str).unwrap_or("0x1001"))?,
        _ => eprintln!("Usage: skynet opcode [list|search|info]"),
    }
    Ok(())
}

fn print_opcode_domains() {
    println!("==========================================================================");
    println!("  SKYNET STANDARDIZED COMPUTE DOMAINS (1,000+ OPCODES)");
    println!("==========================================================================");
    for cat in DOMAIN_CATEGORIES {
        println!(
            "  Base {:#06x} | {:<32} | {}",
            cat.base_code, cat.name, cat.description
        );
    }
}

fn search_opcode_domains(query: &str) {
    if query.is_empty() {
        eprintln!("Usage: skynet opcode search <keyword>");
        return;
    }
    println!("Searching opcodes matching '{query}'...");
    let needle = query.to_lowercase();
    for cat in DOMAIN_CATEGORIES {
        if cat.name.to_lowercase().contains(&needle)
            || cat.description.to_lowercase().contains(&needle)
        {
            println!(
                "  Found Domain {:#06x}: {} ({})",
                cat.base_code, cat.name, cat.description
            );
        }
    }
}

fn print_opcode_info(hex_str: &str) -> Result<(), CliError> {
    let code = u16::from_str_radix(hex_str.trim_start_matches("0x"), 16)
        .map_err(|e| CliError::InvalidHex(e.to_string()))?;
    let desc = lookup_opcode_descriptor(code);
    println!("Opcode Metadata: {:#06x}", desc.opcode);
    println!("  Title       : {}", desc.title);
    println!("  MCP Tool URI: {}", desc.mcp_tool_name);
    println!("  Min VRAM    : {} MB", desc.min_vram_mb);
    println!("  Min RAM     : {} MB", desc.min_ram_mb);
    println!("  Tier Req    : {}", desc.recommended_tier);
    Ok(())
}
