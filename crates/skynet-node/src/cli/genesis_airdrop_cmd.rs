//! `genesis_airdrop_cmd`: Deterministic `genesis` generation/verification and real PoPE
//! hardware-attested `MarketingBootstrapVault` `airdrop claim` CLI handlers.

use std::fs;
use std::path::Path;

use ed25519_dalek::{Signer, SigningKey};
use skynet_cell::pope_benchmark::{
    compute_pope_challenge, verify_pope_proof, PopeChallenge, PopeChallengeParams,
};
use skynet_cell::HardwareProfile;
use skynet_ledger::hwid_airdrop::{HwidClaimRequest, MarketingBootstrapVault};
use skynet_ledger::{
    default_mainnet_genesis, genesis_for_network, NanoSkn, DEFAULT_MAINNET_NETWORK_NAME,
    NANO_PER_SKN,
};

use crate::cli::{
    decode_hex_32, encode_hex, load_or_ephemeral_wallet_key, parse_flag, CliError,
};
use crate::genesis_io::{load_genesis_file, write_genesis_file};

const BOOTSTRAP_RESERVE_SKN: u128 = 1_000_000_000_000;
const DEFAULT_AIRDROP_REWARD_SKN: u128 = 50;
const DEFAULT_HALVING_STEP_DEVICES: u64 = 100_000;

/// Executes `skynet genesis <generate|verify>`.
pub fn handle_genesis(args: &[String]) -> Result<(), CliError> {
    let sub = args.first().map(String::as_str).unwrap_or("help");
    match sub {
        "generate" => generate_genesis_cmd(&args[1..]),
        "verify" => verify_genesis_cmd(&args[1..]),
        _ => Err(CliError::MissingArg(
            "Usage: skynet genesis <generate --output <PATH> [--network <NAME>] | verify --path <PATH>>".into(),
        )),
    }
}

fn generate_genesis_cmd(args: &[String]) -> Result<(), CliError> {
    let out_str = parse_flag(args, "--output")
        .or_else(|| parse_flag(args, "-o"))
        .ok_or_else(|| CliError::MissingArg("--output <PATH>".into()))?;
    let network = parse_flag(args, "--network")
        .unwrap_or_else(|| DEFAULT_MAINNET_NETWORK_NAME.to_string());
    let cfg = match parse_flag(args, "--owner") {
        Some(owner_hex) => genesis_for_network(network, decode_hex_32(&owner_hex)?),
        None if network == DEFAULT_MAINNET_NETWORK_NAME => default_mainnet_genesis(),
        None => {
            let treasury = blake3::derive_key("skynet/genesis/mainnet-seed-treasury/v1", b"SKYNET_V1");
            genesis_for_network(network, treasury)
        }
    };
    let hash = write_genesis_file(Path::new(&out_str), &cfg)?;
    println!("[+] Generated canonical Genesis configuration at: {out_str}");
    println!("    Network      : {}", cfg.network_name);
    println!("    Total Supply : {} nanoSKN (21T SKN: 85% Vault / 15% Circulating)", cfg.total_supply.0);
    println!("    GENESIS_HASH : 0x{}", encode_hex(hash));
    Ok(())
}

fn verify_genesis_cmd(args: &[String]) -> Result<(), CliError> {
    let path_str = parse_flag(args, "--path")
        .or_else(|| parse_flag(args, "--input"))
        .or_else(|| args.first().cloned())
        .ok_or_else(|| CliError::MissingArg("--path <PATH>".into()))?;
    let (cfg, hash) = load_genesis_file(Path::new(&path_str))?;
    println!("[+] Genesis verification PASSED for: {path_str}");
    println!("    Network      : {}", cfg.network_name);
    println!("    Vault (85%)  : {} nanoSKN", cfg.vault_allocation.0);
    println!("    Circ  (15%)  : {} nanoSKN", cfg.circulating_allocation.0);
    println!("    GENESIS_HASH : 0x{}", encode_hex(hash));
    Ok(())
}

/// Executes `skynet airdrop claim [--wallet <PATH>] [--vault <PATH>] [--hwid <HEX>]`.
pub fn handle_airdrop(args: &[String]) -> Result<(), CliError> {
    let sub = args.first().map(String::as_str).unwrap_or("claim");
    if sub != "claim" {
        return Err(CliError::MissingArg("Usage: skynet airdrop claim [--wallet <path>]".into()));
    }
    let wallet_path = parse_flag(args, "--wallet").unwrap_or_else(|| "wallet.json".to_string());
    let (signing_key, human_owner) = load_or_ephemeral_wallet_key(&wallet_path)?;
    let hwid = resolve_hwid_digest(args, &human_owner)?;
    let proof_digest = run_and_verify_pope(&hwid, &human_owner)?;
    let mut vault = load_or_create_marketing_vault(parse_flag(args, "--vault").as_deref())?;
    let req = build_attested_claim(&signing_key, human_owner, hwid, proof_digest);
    let (grant, _output) = vault.claim_reward(req)?;
    if let Some(vault_path) = parse_flag(args, "--vault") {
        save_marketing_vault(Path::new(&vault_path), &vault)?;
    }
    println!("[+] PoPE Hardware Attestation Verified (Digest: 0x{})", encode_hex(&proof_digest[..8]));
    println!("[+] Claim successful! Allocated {} nanoSKN ({} SKN) AgentBond.", grant.0, grant.0 / NANO_PER_SKN);
    Ok(())
}

fn resolve_hwid_digest(args: &[String], owner_pk: &[u8; 32]) -> Result<[u8; 32], CliError> {
    if let Some(hex_hwid) = parse_flag(args, "--hwid") {
        return decode_hex_32(&hex_hwid);
    }
    let cores = std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(4);
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"SKYNET_LOCAL_HWID_FINGERPRINT_V1");
    hasher.update(std::env::consts::OS.as_bytes());
    hasher.update(std::env::consts::ARCH.as_bytes());
    hasher.update(&cores.to_le_bytes());
    hasher.update(owner_pk);
    let mut digest = *hasher.finalize().as_bytes();
    if digest == [0u8; 32] {
        digest[0] = 1;
    }
    Ok(digest)
}

fn run_and_verify_pope(hwid: &[u8; 32], owner_pk: &[u8; 32]) -> Result<[u8; 32], CliError> {
    let device = HardwareProfile::select_compute_device();
    let nonce = *blake3::hash(owner_pk).as_bytes();
    let challenge = PopeChallenge::new(hwid, &nonce, PopeChallengeParams::default());
    let proof = compute_pope_challenge(&device, challenge)?;
    verify_pope_proof(&device, challenge, &proof)?;
    Ok(proof.result_digest)
}

fn build_attested_claim(
    agent_key: &SigningKey,
    human_owner: [u8; 32],
    hwid: [u8; 32],
    proof_digest: [u8; 32],
) -> HwidClaimRequest {
    let agent_pubkey = agent_key.verifying_key().to_bytes();
    let tx_hash = *blake3::hash(&hwid).as_bytes();
    let mut req = HwidClaimRequest::new_simple(hwid, human_owner, agent_pubkey, 1, proof_digest, tx_hash, 0);
    req.attestation_sig = agent_key.sign(&req.signing_digest()).to_bytes().to_vec();
    req
}

fn load_or_create_marketing_vault(path: Option<&str>) -> Result<MarketingBootstrapVault, CliError> {
    if let Some(p) = path {
        if Path::new(p).exists() {
            let raw = fs::read(p)?;
            return Ok(bincode::deserialize(&raw)?);
        }
    }
    let reserve = NanoSkn(BOOTSTRAP_RESERVE_SKN * NANO_PER_SKN);
    let base_reward = NanoSkn(DEFAULT_AIRDROP_REWARD_SKN * NANO_PER_SKN);
    Ok(MarketingBootstrapVault::new(reserve, base_reward, DEFAULT_HALVING_STEP_DEVICES))
}

fn save_marketing_vault(path: &Path, vault: &MarketingBootstrapVault) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, bincode::serialize(vault)?)?;
    Ok(())
}
