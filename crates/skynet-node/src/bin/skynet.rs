//! `skynet`: Standardized User Client, Genesis Verifier, and AI Dispatcher CLI.

use skynet_node::cli::compute_cmd::handle_compute;
use skynet_node::cli::genesis_airdrop_cmd::{handle_airdrop, handle_genesis};
use skynet_node::cli::wallet_opcode_cmd::{handle_opcode, handle_wallet};
use skynet_node::cli::{print_cli_help, CliError};

fn main() {
    if let Err(err) = run_cli() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run_cli() -> Result<(), CliError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args[0] == "help" || args[0] == "--help" || args[0] == "-h" {
        print_cli_help();
        return Ok(());
    }

    match args[0].as_str() {
        "wallet" => handle_wallet(&args[1..]),
        "genesis" => handle_genesis(&args[1..]),
        "airdrop" => handle_airdrop(&args[1..]),
        "opcode" => handle_opcode(&args[1..]),
        "compute" => handle_compute(&args[1..]),
        other => Err(CliError::InvalidValue(format!(
            "Unknown command '{other}'. Use `skynet help` for available commands."
        ))),
    }
}
