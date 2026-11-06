//! Main CLI application entrypoint for `skynet-node`.

use std::collections::HashMap;
use std::path::Path;

use skynet_dna::CellRole;
use skynet_node::{CliCommand, NodeConfig, NodeRuntime, StorageEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = NodeConfig::parse(&raw_args)?;

    match cmd {
        CliCommand::Help => print_help(),
        CliCommand::Status { data_dir } => print_node_status(&data_dir)?,
        CliCommand::Run(cfg) => {
            print_banner(&cfg);
            let mut runtime = NodeRuntime::init(cfg).await?;
            runtime.run().await?;
        }
    }
    Ok(())
}

fn print_banner(cfg: &NodeConfig) {
    println!("==========================================================================");
    println!("  SKYNET BIO-CELLULAR AI BLOCKCHAIN — NODE DAEMON");
    println!("==========================================================================");
    println!("  Node Name : {}", cfg.name);
    println!("  Tier      : {:?}", cfg.tier);
    println!("  Data Dir  : {}", cfg.data_dir.display());
    println!("  UDP Port  : {}", cfg.port);
    println!("--------------------------------------------------------------------------\n");
}

fn print_help() {
    println!("SkyNet Bio-Cellular AI Node Daemon");
    println!("\nUSAGE:");
    println!("  skynet-node [COMMAND] [OPTIONS]");
    println!("\nCOMMANDS:");
    println!("  run (default)     Start the daemon listener and biological neural loop");
    println!("  status            Query and display local node status from disk");
    println!("  help, --help, -h  Display this help menu");
    println!("\nOPTIONS:");
    println!("  --port, -p <u16>       UDP P2P port to bind (default: 8888)");
    println!("  --data-dir, -d <path>  Storage directory (default: ~/.skynet/data)");
    println!("  --name, -n <str>       Node organism name");
    println!("  --tier, -t <tier>      Hardware tier: TINY, STANDARD, COLOSSAL");
    println!("  --seed, -s <ip:port>   Bootstrap peer address (repeatable)");
    println!("  --owner <hex>          Owner HumanWallet public key (32 bytes hex)");
}

fn print_node_status(data_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let storage = StorageEngine::new(data_dir)?;
    if !storage.has_existing_state() {
        println!("  [!] No existing SkyNet state found at {}", data_dir.display());
        println!("      Run `skynet-node run --data-dir {}` to initialize this node.", data_dir.display());
        return Ok(());
    }

    let meta = storage.load_metadata()?.unwrap();
    let org = storage.load_organism()?.unwrap();
    let ledger = storage.load_ledger()?.unwrap();
    let peers = storage.load_peers()?;

    let mut role_counts: HashMap<CellRole, usize> = HashMap::new();
    for c in &org.cells {
        *role_counts.entry(c.role()).or_default() += 1;
    }

    println!("==========================================================================");
    println!("  SKYNET NODE LOCAL INSPECTION: {}", meta.name);
    println!("==========================================================================");
    println!("  Identity (Pubkey) : {}", meta.pubkey.iter().map(|b| format!("{:02x}", b)).collect::<String>());
    println!("  Hardware Tier     : {:?}", meta.tier);
    println!("  Epoch / Pulse Seq : {} / {}", meta.epoch, meta.pulse_seq);
    println!("  On-Chain Balance  : {}", ledger.balance_of(&meta.pubkey));
    println!("  Blood Pool Energy : {}", org.blood_pool);
    println!("  Voucher Earned    : {} nanoSKN", org.total_voucher_nano_earned.0);
    println!("  Living AI Cells   : {} cells total", org.cells.len());
    for (role, count) in role_counts {
        println!("    - {:<10} : {} cells", format!("{:?}", role), count);
    }
    println!("  Routing Peers     : {} known peers", peers.len());
    println!("--------------------------------------------------------------------------");
    Ok(())
}
