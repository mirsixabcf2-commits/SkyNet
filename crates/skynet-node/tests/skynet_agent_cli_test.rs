//! Integration test suite for `skynet-agent` worker daemon CLI.

use std::fs;
use std::process::Command;

#[test]
fn test_skynet_agent_cli_help() {
    let bin_path = env!("CARGO_BIN_EXE_skynet-agent");

    let output = Command::new(bin_path)
        .arg("--help")
        .output()
        .expect("Failed to execute skynet-agent --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SkyNet Bio-Cellular AI Worker Daemon CLI"));
    assert!(stdout.contains("--auto-bid"));
    assert!(stdout.contains("--init-worker"));
}

#[test]
fn test_skynet_agent_init_worker() {
    let bin_path = env!("CARGO_BIN_EXE_skynet-agent");
    let test_dir = std::env::temp_dir().join("skynet_agent_test_dir");
    let _ = fs::create_dir_all(&test_dir);

    let output = Command::new(bin_path)
        .current_dir(&test_dir)
        .args(["--init-worker", "--wallet-path", "custom_wallet.json"])
        .output()
        .expect("Failed to execute --init-worker");
    assert!(output.status.success());

    let profile_file = test_dir.join("worker_profile.json");
    assert!(profile_file.exists());

    let content = fs::read_to_string(&profile_file).expect("Read profile failed");
    let profile: serde_json::Value = serde_json::from_str(&content).expect("Valid JSON");
    assert!(profile.get("cpu_cores").and_then(|v| v.as_u64()).is_some());
    assert!(profile.get("default_tags").and_then(|v| v.as_array()).is_some());

    let _ = fs::remove_file(&profile_file);
    let _ = fs::remove_dir_all(&test_dir);
}
