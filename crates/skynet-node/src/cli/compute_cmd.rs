//! `compute_cmd`: Turnkey presets, real UDP `compute dispatch`, and Fountain-coded
//! `compute train` shard streaming CLI handlers.

use std::fs;
use std::net::{SocketAddr, UdpSocket};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ed25519_dalek::SigningKey;
use skynet_cell::{encode_dense_f16, MicroTrainBatch};
use skynet_ledger::NANO_PER_SKN;
use skynet_wire::{
    encode_micro_train_shard, ComputeMessage, ComputeTaskManifest, EpidermalFilter, ImpulseBody,
    ImpulsePacket, MicroBatchManifest, MicroTrainWireMessage, WireError,
};

use crate::cli::{encode_hex, has_flag, load_or_ephemeral_wallet_key, parse_flag, CliError};
use crate::genesis_io::load_genesis_file;
use crate::preprocessor::{run_preprocessing, to_candle_task};
use crate::presets::{default_options_for_preset, find_preset, list_presets};

const DEFAULT_WORKER_TARGET: &str = "127.0.0.1:8889";
const TRAIN_DIM_IN: usize = 8;
const TRAIN_DIM_HIDDEN: usize = 16;
const TRAIN_DIM_OUT: usize = 4;
const DEFAULT_INNER_STEPS: u16 = 6;
const DEFAULT_LR_MICRO: u32 = 15_000;
const OPCODE_MICRO_TRAIN_MLP: u16 = 0x1301;

struct UdpSendContext {
    socket: UdpSocket,
    target: SocketAddr,
    signing_key: SigningKey,
    genesis_hash: Option<[u8; 32]>,
    seq: u32,
}

impl UdpSendContext {
    fn from_args(args: &[String]) -> Result<Self, CliError> {
        let wallet_path = parse_flag(args, "--wallet").unwrap_or_else(|| "wallet.json".to_string());
        let (signing_key, _) = load_or_ephemeral_wallet_key(&wallet_path)?;
        let target_str = parse_flag(args, "--target").unwrap_or_else(|| DEFAULT_WORKER_TARGET.to_string());
        let target: SocketAddr = target_str
            .parse()
            .map_err(|e: std::net::AddrParseError| CliError::InvalidValue(e.to_string()))?;
        let genesis_hash = match parse_flag(args, "--genesis") {
            Some(p) => Some(load_genesis_file(Path::new(&p))?.1),
            None => None,
        };
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(Duration::from_millis(80)))?;
        Ok(Self { socket, target, signing_key, genesis_hash, seq: 0 })
    }

    fn send_body(&mut self, hop_ttl: u8, body: ImpulseBody) -> Result<usize, CliError> {
        self.seq = self.seq.wrapping_add(1);
        let pkt = ImpulsePacket::new_signed(&self.signing_key, 0, 0, hop_ttl, 1, self.seq, body)?;
        let bytes = match self.genesis_hash {
            Some(gh) => EpidermalFilter::encode_framed_packet(&pkt, &gh, unix_now_sec())?,
            None => pkt.encode_datagram()?,
        };
        Ok(self.socket.send_to(&bytes, self.target)?)
    }
}

/// Executes `skynet compute <presets|dispatch|train>`.
pub fn handle_compute(args: &[String]) -> Result<(), CliError> {
    let sub = args.first().map(String::as_str).unwrap_or("presets");
    match sub {
        "presets" => print_presets(),
        "dispatch" => dispatch_task_udp(&args[1..])?,
        "train" => stream_micro_train_udp(&args[1..])?,
        _ => eprintln!("Usage: skynet compute [presets|dispatch|train]"),
    }
    Ok(())
}

fn print_presets() {
    println!("==========================================================================");
    println!("  SKYNET TURNKEY PREPROCESSING PRESETS");
    println!("==========================================================================");
    for p in list_presets() {
        println!(
            "  * ID: {:<22} | Opcode: {:#06x} | Bounty: {} SKN",
            p.id, p.default_opcode, p.suggested_bounty_skn
        );
        println!("    Title: {}", p.title);
        println!("    Desc : {}\n", p.description);
    }
}

fn dispatch_task_udp(args: &[String]) -> Result<(), CliError> {
    let input_path = parse_flag(args, "--input").unwrap_or_else(|| "data.txt".to_string());
    let preset_name = parse_flag(args, "--preset").unwrap_or_else(|| "text-audit".to_string());
    let preset = find_preset(&preset_name)
        .ok_or_else(|| CliError::InvalidValue(format!("Unknown preset: {preset_name}")))?;
    let content = load_input_or_demo(&input_path, preset.id)?;
    let mut opts = default_options_for_preset(preset.id);
    if has_flag(args, "--encrypt") {
        opts.encrypt_envelope = true;
    }
    let result = run_preprocessing(&content, &opts).map_err(CliError::Preprocess)?;
    let task = to_candle_task(&result.tensor_floats, 16, 32, 4);
    let mut ctx = UdpSendContext::from_args(args)?;
    let payload_bytes = bincode::serialize(&task).map_err(WireError::Codec)?;
    let manifest = build_compute_manifest(&ctx.signing_key, preset.default_opcode, preset.suggested_bounty_skn, &payload_bytes);
    let sent_ann = ctx.send_body(4, ImpulseBody::Compute(ComputeMessage::TaskAnnouncement(manifest)))?;
    let sent_pay = ctx.send_body(4, ImpulseBody::Compute(ComputeMessage::DirectPayload {
        task_id: task.task_id,
        is_result: false,
        data: payload_bytes,
    }))?;
    println!("[+] Assembled Candle Task: ID 0x{}", encode_hex(&task.task_id[..8]));
    println!("[+] Dispatched UDP TaskAnnouncement ({sent_ann}B) + DirectPayload ({sent_pay}B) to {}!", ctx.target);
    try_collect_udp_ack(&ctx.socket);
    Ok(())
}

fn build_compute_manifest(key: &SigningKey, opcode: u16, bounty_skn: u64, raw_payload: &[u8]) -> ComputeTaskManifest {
    let bounty_nano = u128::from(bounty_skn) * NANO_PER_SKN;
    let input_hash = *blake3::hash(raw_payload).as_bytes();
    ComputeTaskManifest {
        task_id: input_hash,
        client_pubkey: key.verifying_key().to_bytes(),
        opcode,
        required_tier: 1,
        input_hash,
        timeout_epoch: 100,
        max_ticks: 50_000,
        bounty_nano_skn: bounty_nano,
        min_worker_bond_nano_skn: bounty_nano / 5,
        auditor_bounty_nano_skn: bounty_nano / 10,
        required_auditors: 1,
    }
}

fn stream_micro_train_udp(args: &[String]) -> Result<(), CliError> {
    let input_path = parse_flag(args, "--input")
        .ok_or_else(|| CliError::MissingArg("--input <PATH>".into()))?;
    let epochs: u16 = parse_flag(args, "--epochs")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
        .max(1);
    let bounty_skn: u64 = parse_flag(args, "--bounty")
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let raw_bytes = fs::read(&input_path)?;
    let mut ctx = UdpSendContext::from_args(args)?;
    let task_id = *blake3::hash(&raw_bytes).as_bytes();
    let bounty_chunk = (u128::from(bounty_skn) * NANO_PER_SKN) / u128::from(epochs);

    let mut total_droplets = 0usize;
    for shard_id in 0..epochs {
        let batch = build_training_batch(task_id, shard_id, &raw_bytes);
        let shard_bytes = bincode::serialize(&batch).map_err(WireError::Codec)?;
        let manifest = build_micro_manifest(
            ctx.signing_key.verifying_key().to_bytes(),
            (task_id, shard_id, epochs),
            &shard_bytes,
            bounty_chunk,
        );
        let _ = ctx.send_body(4, ImpulseBody::MicroTrain(MicroTrainWireMessage::MicroBatchManifest(manifest)))?;
        total_droplets += send_shard_droplets(&mut ctx, &shard_bytes)?;
    }
    println!("[+] Streamed {epochs} Micro-Training shard(s) ({total_droplets} FountainDroplets) to {}", ctx.target);
    try_collect_udp_ack(&ctx.socket);
    Ok(())
}

fn build_training_batch(task_id: [u8; 32], shard_id: u16, raw: &[u8]) -> MicroTrainBatch {
    let weight_count = TRAIN_DIM_IN * TRAIN_DIM_HIDDEN + TRAIN_DIM_HIDDEN + TRAIN_DIM_HIDDEN * TRAIN_DIM_OUT + TRAIN_DIM_OUT;
    let init_w: Vec<f32> = (0..weight_count).map(|i| ((i as f32) * 0.03).sin() * 0.2).collect();
    let sample_count = 4usize;
    let samples_x: Vec<f32> = (0..sample_count * TRAIN_DIM_IN)
        .map(|i| {
            let b = raw.get((i + usize::from(shard_id) * 13) % raw.len().max(1)).copied().unwrap_or(64);
            (f32::from(b) / 127.5) - 1.0
        })
        .collect();
    let targets_y: Vec<f32> = (0..sample_count * TRAIN_DIM_OUT)
        .map(|i| samples_x[i % samples_x.len()] * 0.5)
        .collect();
    MicroTrainBatch { task_id, shard_id, base_weights_f16: encode_dense_f16(&init_w), samples_x, targets_y }
}

fn build_micro_manifest(
    client_pubkey: [u8; 32],
    shard_meta: ([u8; 32], u16, u16),
    shard_bytes: &[u8],
    bounty_per_chunk_nano: u128,
) -> MicroBatchManifest {
    MicroBatchManifest {
        task_id: shard_meta.0,
        client_pubkey,
        opcode: OPCODE_MICRO_TRAIN_MLP,
        shard_id: shard_meta.1,
        total_shards: shard_meta.2,
        dim_in: TRAIN_DIM_IN as u16,
        dim_hidden: TRAIN_DIM_HIDDEN as u16,
        dim_out: TRAIN_DIM_OUT as u16,
        inner_steps: DEFAULT_INNER_STEPS,
        learning_rate_micro: DEFAULT_LR_MICRO,
        shard_object_hash: *blake3::hash(shard_bytes).as_bytes(),
        shard_bytes_len: shard_bytes.len() as u32,
        bounty_per_chunk_nano,
        min_worker_bond_nano: bounty_per_chunk_nano / 5,
    }
}

fn send_shard_droplets(ctx: &mut UdpSendContext, shard_bytes: &[u8]) -> Result<usize, CliError> {
    let mut enc = encode_micro_train_shard(shard_bytes)?;
    let k = usize::from(enc.num_blocks()).max(1);
    let droplet_budget = k + (k / 2).max(6);
    for _ in 0..droplet_budget {
        let _ = ctx.send_body(4, ImpulseBody::FountainDroplet(enc.next_droplet()))?;
    }
    Ok(droplet_budget)
}

fn load_input_or_demo(input_path: &str, preset_id: &str) -> Result<String, CliError> {
    if Path::new(input_path).exists() {
        return Ok(fs::read_to_string(input_path)?);
    }
    Ok(format!("Demo raw content for preset {preset_id} with 0912345678 and email demo@skynet.ai"))
}

fn try_collect_udp_ack(socket: &UdpSocket) {
    let mut buf = [0u8; 1500];
    if let Ok((len, peer)) = socket.recv_from(&mut buf) {
        println!("[+] Received {len}B UDP response from worker {peer}");
    }
}

fn unix_now_sec() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
