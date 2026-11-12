//! `skynet-agent`: Real UDP Opportunistic Edge Worker Daemon powered by
//! `OsTelemetryGovernor`, `OpportunisticWorkerEngine`, and `CheckpointStore`.

use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use skynet_agent::{
    AgentNode, OpportunisticWorkerEngine, PowerProfitabilityGuard, SnowballParams,
    SEED_KEEPALIVE_INTERVAL_SEC,
};
use skynet_cell::HardwareTier;
use skynet_ledger::NANO_PER_SKN;
use skynet_node::cli::encode_hex;
use skynet_node::{
    bootstrap_node_ledger, CheckpointStore, NodeIdentity, OsTelemetryGovernor, RuntimeP2pDriver,
    WorkerCliCommand, WorkerCliConfig, WorkerStudioStateSnapshot, LAN_BEACON_INTERVAL_MS,
};
use tokio::net::UdpSocket;
use tokio::time::interval;

const STUDIO_FLUSH_INTERVAL_MS: u64 = 500;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let res = match WorkerCliConfig::parse(&args) {
        Ok(WorkerCliCommand::Help) => {
            print_help();
            Ok(())
        }
        Ok(WorkerCliCommand::InitWorker(cfg)) => init_worker_profile(&cfg),
        Ok(WorkerCliCommand::Run(cfg)) => run_worker_daemon(*cfg).await,
        Err(err) => Err(Box::new(err) as Box<dyn std::error::Error>),
    };
    if let Err(err) = res {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn print_help() {
    println!("SkyNet Bio-Cellular AI Worker Daemon CLI\n");
    println!("USAGE:");
    println!("  skynet-agent [OPTIONS]\n");
    println!("OPTIONS:");
    println!("  --init-worker                  Auto-detect hardware and initialize worker profile");
    println!("  --opportunistic                Zero-impact gamer/edge mode (<100ms preempt SLA)");
    println!("  --owner <HEX_PUBKEY>           Cold-wallet HumanWallet owner pubkey for Yield Sweep");
    println!("  --seed, -s <ip:port>           Seed rendezvous peer address (repeatable)");
    println!("  --genesis <path>               Path to canonical genesis.json network file");
    println!("  --schedule <HH:MM-HH:MM>       Allowed daily active compute window");
    println!("  --auto-sweep-skn <u64>         Auto-sweep threshold to cold wallet (default: 50)");
    println!("  --game-watchlist <csv>         Extra process names triggering instant preemption");
    println!("  --max-vram-pct <u8>            VRAM ceiling percentage (default: 50)");
    println!("  --idle-hold-secs <u64>         Continuous idle hold before resume (default: 30)");
    println!("  --listen, -p <port>            UDP listening port (default: 8889)");
    println!("  --auto-bid                     Enable automated task discovery and bidding");
    println!("  --tags <tag1,tag2>             Comma-separated compute capability tags");
    println!("  --min-bounty-rate <rate>       Minimum acceptable NanoSkn rate floor");
    println!("  --battery-guard                Pause computations when on battery power");
    println!("  --temp-limit <celsius>         Thermal cutoff threshold (default: 78)");
    println!("  --wallet-path <path>           Path to beneficiary wallet JSON (default: wallet.json)");
    println!("  --data-dir <path>              Checkpoint & studio telemetry dir (default: .skynet_worker)");
}

fn init_worker_profile(cfg: &WorkerCliConfig) -> Result<(), Box<dyn std::error::Error>> {
    let cpu_cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let profile = serde_json::json!({
        "cpu_cores": cpu_cores,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "default_tags": ["cpu-general", "data-etl", "candle-inference", "micro-train"],
        "min_bounty_rate": cfg.min_bounty_rate,
        "battery_guard": cfg.battery_guard,
        "temp_limit_c": cfg.temp_limit_c,
        "wallet_path": cfg.wallet_path,
    });
    fs::write("worker_profile.json", serde_json::to_string_pretty(&profile)?)?;
    println!("[+] Initialized worker profile saved to 'worker_profile.json' ({cpu_cores} cores).");
    Ok(())
}

struct WorkerLoopCtx {
    cfg: WorkerCliConfig,
    store: CheckpointStore,
    socket: UdpSocket,
    p2p: RuntimeP2pDriver,
    agent: AgentNode,
    gov: OsTelemetryGovernor,
    stop_flag: Arc<AtomicBool>,
}

async fn run_worker_daemon(cfg: WorkerCliConfig) -> Result<(), Box<dyn std::error::Error>> {
    let store = CheckpointStore::new(&cfg.data_dir)?;
    let identity = NodeIdentity::load_or_generate(&cfg.data_dir.join("worker_identity.key"))?;
    let (ledger, genesis_hash) = bootstrap_node_ledger(cfg.genesis_path.as_deref(), identity.pubkey)?;
    let mut agent = build_worker_agent(&cfg, &identity, ledger)?;

    let socket = UdpSocket::bind(("0.0.0.0", cfg.listen_port)).await?;
    let actual_port = socket.local_addr()?.port();
    let mut p2p = RuntimeP2pDriver::new(genesis_hash, actual_port, &mut agent);

    let stop_flag = Arc::new(AtomicBool::new(false));
    let gov = OsTelemetryGovernor::new_host(
        cfg.to_governor_config(),
        cfg.game_watchlist.clone(),
        agent.opportunistic_worker.guard.clone(),
    );
    let _gov_handle = gov.spawn_loop(Arc::clone(&stop_flag));

    println!("[+] SkyNet Opportunistic Worker bound at 0.0.0.0:{actual_port} (0x{})", encode_hex(&identity.pubkey[..8]));
    let _ = p2p.send_seed_keepalives(&socket, &mut agent, &cfg.seed_peers).await?;

    run_worker_loop(WorkerLoopCtx { cfg, store, socket, p2p, agent, gov, stop_flag }).await
}

async fn run_worker_loop(mut ctx: WorkerLoopCtx) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let mut buf = [0u8; 2048];
    let mut seed_timer = interval(Duration::from_secs(SEED_KEEPALIVE_INTERVAL_SEC));
    let mut lan_timer = interval(Duration::from_millis(LAN_BEACON_INTERVAL_MS));
    let mut studio_timer = interval(Duration::from_millis(STUDIO_FLUSH_INTERVAL_MS));

    loop {
        tokio::select! {
            recv_res = ctx.socket.recv_from(&mut buf) => {
                let (len, src) = recv_res?;
                let _ = ctx.p2p.process_datagram(&ctx.socket, &mut ctx.agent, (&buf[..len], src)).await?;
            }
            _ = seed_timer.tick() => {
                let _ = ctx.p2p.send_seed_keepalives(&ctx.socket, &mut ctx.agent, &ctx.cfg.seed_peers).await?;
            }
            _ = lan_timer.tick() => {
                let _ = ctx.p2p.broadcast_lan_beacon(&ctx.socket, &mut ctx.agent).await?;
            }
            _ = studio_timer.tick() => {
                flush_studio_telemetry(&ctx.store, &ctx.gov, &mut ctx.agent, started.elapsed().as_secs_f64())?;
            }
            _ = tokio::signal::ctrl_c() => {
                ctx.stop_flag.store(true, Ordering::Relaxed);
                break;
            }
        }
    }
    Ok(())
}

fn build_worker_agent(
    cfg: &WorkerCliConfig,
    identity: &NodeIdentity,
    ledger: skynet_ledger::UtxoDagLedger,
) -> Result<AgentNode, skynet_agent::AgentError> {
    let mut agent = AgentNode::from_signing_key(
        "Skynet-Opportunistic-Worker",
        2026,
        identity.signing_key.clone(),
        HardwareTier::StandardNode,
        ledger,
        SnowballParams::default(),
    )?;
    let floor_nano = (cfg.min_bounty_rate.max(0.0) * 1_000.0) as u128;
    let mut engine = OpportunisticWorkerEngine::new(
        cfg.to_governor_config(),
        PowerProfitabilityGuard::with_floor_nano_per_sec(floor_nano),
    );
    if let Some(sweep_cfg) = cfg.to_yield_sweep_config() {
        engine = engine.with_sweep_config(sweep_cfg);
    }
    agent.opportunistic_worker = engine;
    Ok(agent)
}

fn flush_studio_telemetry(
    store: &CheckpointStore,
    gov: &OsTelemetryGovernor,
    agent: &mut AgentNode,
    uptime_secs: f64,
) -> Result<(), skynet_node::CheckpointStoreError> {
    let snap = gov.latest_snapshot();
    let st = agent.opportunistic_worker.update_telemetry(&snap);
    let rep = agent.compute_coordinator.escrow_vault.worker_reputation(&agent.pubkey);
    let earned = agent.opportunistic_worker.cumulative_earned_nano.0.max(agent.total_voucher_nano_earned.0);
    let swept = agent
        .opportunistic_worker
        .sweep_cfg
        .as_ref()
        .map(|s| agent.ledger.balance_of(&s.human_owner).0)
        .unwrap_or(0);
    let hrs = (uptime_secs / 3_600.0).max(1.0 / 3_600.0);
    let rate_skn_hr = (earned as f64 / NANO_PER_SKN as f64) / hrs;

    let studio = WorkerStudioStateSnapshot {
        governor_state: format!("{st:?}"),
        preempt_latency_ms: gov.last_preempt_latency_ms(),
        gpu_util_pct: snap.gpu_util_pct,
        cpu_util_pct: snap.cpu_util_pct,
        vram_used_pct: snap.vram_used_pct,
        temp_c: snap.temp_c,
        on_battery: snap.on_battery,
        active_game_detected: snap.active_game_detected,
        fountain_shards_completed: agent.opportunistic_worker.completed_shards,
        pouw_score_bps: rep.pouw_score_bps,
        slashed_count: rep.slashed_count,
        earned_nano_skn: earned,
        swept_to_cold_nano_skn: swept,
        earning_rate_skn_per_hr: rate_skn_hr,
        updated_at_sec: snap.observed_at_sec,
    };
    let _ = store.write_studio_state(&studio)?;
    Ok(())
}
