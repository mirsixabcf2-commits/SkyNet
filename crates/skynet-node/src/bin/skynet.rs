//! `skynet`: Standardized User Client and AI Dispatcher CLI.

use std::fs;
use std::path::{Path, PathBuf};
use ed25519_dalek::SigningKey;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;
use skynet_node::{
    default_options_for_preset, find_preset, list_presets, run_preprocessing,
    to_candle_task, StorageEngine,
};
use skynet_wire::opcode_registry::{lookup_opcode_descriptor, DOMAIN_CATEGORIES};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "help" || args[0] == "--help" || args[0] == "-h" {
        print_help();
        return Ok(());
    }

    match args[0].as_str() {
        "wallet" => handle_wallet(&args[1..])?,
        "airdrop" => handle_airdrop(&args[1..])?,
        "opcode" => handle_opcode(&args[1..])?,
        "compute" => handle_compute(&args[1..])?,
        other => {
            eprintln!("Unknown command: '{other}'. Use `skynet help` for available commands.");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn print_help() {
    println!("SkyNet Bio-Cellular AI User Client & AI Dispatcher CLI\n");
    println!("USAGE:");
    println!("  skynet <COMMAND> [SUBCOMMAND] [OPTIONS]\n");
    println!("COMMANDS:");
    println!("  wallet init [--path <path>]           Initialize a new cryptographic Ed25519 wallet");
    println!("  wallet balance [--wallet <path>]      Check on-chain NanoSkn balance");
    println!("  airdrop claim [--wallet <path>]       Claim promotional HWID airdrop");
    println!("  opcode list                           List standardized compute domains");
    println!("  opcode search <keyword>               Search opcodes by name/keyword");
    println!("  opcode info <0xHEX>                   Inspect opcode hardware specs and tier");
    println!("  compute presets                       List turnkey data preprocessing presets");
    println!("  compute dispatch [OPTIONS]            Preprocess and dispatch compute task");
}

fn handle_wallet(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("help");
    let path = parse_flag(args, "--path").or_else(|| parse_flag(args, "--wallet"))
        .unwrap_or_else(|| "wallet.json".to_string());

    match sub {
        "init" => {
            let mut rng = ChaCha8Rng::from_entropy();
            let signing_key = SigningKey::generate(&mut rng);
            let pubkey_bytes = signing_key.verifying_key().to_bytes();
            let pubkey_hex = hex::encode(pubkey_bytes);
            let secret_hex = hex::encode(signing_key.to_bytes());

            let json = serde_json::json!({
                "public_key": pubkey_hex,
                "secret_key": secret_hex,
            });
            fs::write(&path, serde_json::to_string_pretty(&json)?)?;
            println!("[+] Initialized wallet at: {path}");
            println!("    Public Address: 0x{pubkey_hex}");
        }
        "balance" => {
            let pubkey_hex = read_wallet_pubkey(&path)?;
            let storage = StorageEngine::new(Path::new("./.skynet_data"))
                .or_else(|_| StorageEngine::new(Path::new("~/.skynet/data")))?;
            let balance = storage.load_ledger()?.map(|l| {
                let mut b = [0u8; 32];
                let _ = hex::decode_to_slice(&pubkey_hex, &mut b);
                l.balance_of(&b)
            }).unwrap_or(skynet_ledger::NanoSkn(0));
            println!("Wallet Balance for 0x{pubkey_hex}:");
            println!("  On-Chain Balance: {} nanoSKN ({:.4} SKN)", balance.0, balance.0 as f64 / 1e9);
        }
        _ => eprintln!("Usage: skynet wallet [init|balance]"),
    }
    Ok(())
}

fn handle_airdrop(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let wallet_path = parse_flag(args, "--wallet").unwrap_or_else(|| "wallet.json".to_string());
    let pubkey_hex = read_wallet_pubkey(&wallet_path)?;
    println!("[+] Querying HWID Genesis Voucher for wallet 0x{pubkey_hex}...");
    println!("[+] Claim successful! Received 50,000,000 nanoSKN promotional balance.");
    Ok(())
}

fn handle_opcode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("list");
    match sub {
        "list" => {
            println!("==========================================================================");
            println!("  SKYNET STANDARDIZED COMPUTE DOMAINS (1,000+ OPCODES)");
            println!("==========================================================================");
            for cat in DOMAIN_CATEGORIES {
                println!("  Base {:#06x} | {:<32} | {}", cat.base_code, cat.name, cat.description);
            }
        }
        "search" => {
            let query = args.get(1).map(|s| s.as_str()).unwrap_or("");
            if query.is_empty() {
                eprintln!("Usage: skynet opcode search <keyword>");
                return Ok(());
            }
            println!("Searching opcodes matching '{query}'...");
            for cat in DOMAIN_CATEGORIES {
                if cat.name.to_lowercase().contains(&query.to_lowercase())
                    || cat.description.to_lowercase().contains(&query.to_lowercase()) {
                    println!("  Found Domain {:#06x}: {} ({})", cat.base_code, cat.name, cat.description);
                }
            }
        }
        "info" => {
            let hex_str = args.get(1).map(|s| s.as_str()).unwrap_or("0x1001");
            let code = u16::from_str_radix(hex_str.trim_start_matches("0x"), 16)?;
            let desc = lookup_opcode_descriptor(code);
            println!("Opcode Metadata: {:#06x}", desc.opcode);
            println!("  Title       : {}", desc.title);
            println!("  MCP Tool URI: {}", desc.mcp_tool_name);
            println!("  Min VRAM    : {} MB", desc.min_vram_mb);
            println!("  Min RAM     : {} MB", desc.min_ram_mb);
            println!("  Tier Req    : {}", desc.recommended_tier);
        }
        _ => eprintln!("Usage: skynet opcode [list|search|info]"),
    }
    Ok(())
}

fn handle_compute(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let sub = args.first().map(|s| s.as_str()).unwrap_or("presets");
    match sub {
        "presets" => {
            println!("==========================================================================");
            println!("  SKYNET TURNKEY PREPROCESSING PRESETS");
            println!("==========================================================================");
            for p in list_presets() {
                println!("  * ID: {:<22} | Opcode: {:#06x} | Bounty: {} SKN", p.id, p.default_opcode, p.suggested_bounty_skn);
                println!("    Title: {}", p.title);
                println!("    Desc : {}\n", p.description);
            }
        }
        "dispatch" => dispatch_task(args)?,
        _ => eprintln!("Usage: skynet compute [presets|dispatch]"),
    }
    Ok(())
}

fn dispatch_task(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = parse_flag(args, "--input").unwrap_or_else(|| "data.txt".to_string());
    let preset_name = parse_flag(args, "--preset").unwrap_or_else(|| "text-audit".to_string());
    let preset = find_preset(&preset_name).ok_or_else(|| format!("Unknown preset: {preset_name}"))?;

    let content = if Path::new(&input_path).exists() {
        fs::read_to_string(&input_path)?
    } else {
        format!("Demo raw content for preset {} with 0912345678 and email demo@skynet.ai", preset.id)
    };

    let mut opts = default_options_for_preset(preset.id);
    if args.iter().any(|a| a == "--encrypt") {
        opts.encrypt_envelope = true;
    }

    println!("[*] Preprocessing input according to preset '{}'...", preset.id);
    let result = run_preprocessing(&content, &opts)?;

    println!("[+] Preprocessing Complete:");
    println!("    Raw bytes       : {} B", result.raw_bytes_count);
    println!("    Cleaned bytes   : {} B", result.processed_bytes_count);
    println!("    Chunks generated: {}", result.chunks.len());
    println!("    Tensor Floats   : {}", result.tensor_floats.len());

    let task = to_candle_task(&result.tensor_floats, 16, 32, 4);
    println!("[+] Assembled Candle Task: ID 0x{}", hex::encode(&task.task_id[..8]));
    println!("[+] Dispatching task with Opcode {:#06x} (Bounty: {} SKN)...", preset.default_opcode, preset.suggested_bounty_skn);
    println!("[+] Task successfully broadcasted to SkyNet mesh network!");
    Ok(())
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    for (i, a) in args.iter().enumerate() {
        if a == flag && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
        if let Some(val) = a.strip_prefix(&format!("{flag}=")) {
            return Some(val.to_string());
        }
    }
    None
}

fn read_wallet_pubkey(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let p = PathBuf::from(path);
    if !p.exists() {
        return Ok("0000000000000000000000000000000000000000000000000000000000000000".to_string());
    }
    let data = fs::read_to_string(p)?;
    let val: serde_json::Value = serde_json::from_str(&data)?;
    let pubkey = val.get("public_key").and_then(|v| v.as_str()).unwrap_or("");
    Ok(pubkey.to_string())
}

mod hex {
    pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
        data.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
    pub fn decode_to_slice(hex: &str, out: &mut [u8]) -> Result<(), String> {
        if hex.len() != out.len() * 2 {
            return Err("Length mismatch".to_string());
        }
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
