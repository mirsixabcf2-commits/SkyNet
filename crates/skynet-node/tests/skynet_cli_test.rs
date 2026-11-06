//! Integration test suite for `skynet` CLI binary and preprocessor pipelines.

use std::fs;
use std::process::Command;
use skynet_node::{
    default_options_for_preset, find_preset, list_presets, run_preprocessing,
    to_candle_task,
};

#[test]
fn test_presets_catalog_and_options() {
    let presets = list_presets();
    assert!(presets.len() >= 5);

    let audit_preset = find_preset("text-audit").expect("text-audit preset must exist");
    assert_eq!(audit_preset.default_opcode, 0x0215);

    let opts = default_options_for_preset(audit_preset.id);
    assert!(opts.redact_pii);
    assert!(opts.encrypt_envelope);
}

#[test]
fn test_preprocessor_pipeline_and_tensor_wrap() {
    let raw_text = "Thanh toan cho 0912345678 tai hop dong so 001201099887 kem email boss@skynet.org.";
    let opts = default_options_for_preset("text-audit");

    let result = run_preprocessing(raw_text, &opts).expect("Preprocessing should succeed");
    let processed = result.processed_text.as_deref().unwrap_or("");

    assert!(processed.contains("[REDACTED_PHONE]"));
    assert!(processed.contains("[REDACTED_ID]"));
    assert!(processed.contains("[REDACTED_EMAIL]"));
    assert!(result.envelope_bytes.is_some());
    assert!(result.encryption_key_hex.is_some());

    let task = to_candle_task(&result.tensor_floats, 8, 16, 2);
    assert_eq!(task.dim_in, 8);
    assert_eq!(task.dim_hidden, 16);
    assert_eq!(task.dim_out, 2);
    assert_eq!(task.input.len(), 8);
}

#[test]
fn test_skynet_cli_binary_invocation() {
    let bin_path = env!("CARGO_BIN_EXE_skynet");

    // 1. Help subcommand
    let help_output = Command::new(bin_path)
        .arg("help")
        .output()
        .expect("Failed to execute skynet binary");
    assert!(help_output.status.success());
    let stdout = String::from_utf8_lossy(&help_output.stdout);
    assert!(stdout.contains("SkyNet Bio-Cellular AI User Client"));

    // 2. Opcode list subcommand
    let opcode_output = Command::new(bin_path)
        .args(["opcode", "list"])
        .output()
        .expect("Failed to execute opcode list");
    assert!(opcode_output.status.success());
    let opcode_stdout = String::from_utf8_lossy(&opcode_output.stdout);
    assert!(opcode_stdout.contains("STANDARDIZED COMPUTE DOMAINS"));

    // 3. Compute presets subcommand
    let presets_output = Command::new(bin_path)
        .args(["compute", "presets"])
        .output()
        .expect("Failed to execute compute presets");
    assert!(presets_output.status.success());
    let presets_stdout = String::from_utf8_lossy(&presets_output.stdout);
    assert!(presets_stdout.contains("text-audit"));
    assert!(presets_stdout.contains("csv-clean-normalize"));

    // 4. Wallet init temporary file
    let tmp_dir = std::env::temp_dir().join("skynet_test_wallet_dir");
    let _ = fs::create_dir_all(&tmp_dir);
    let tmp_wallet = tmp_dir.join("test_wallet.json");

    let wallet_output = Command::new(bin_path)
        .args(["wallet", "init", "--path", tmp_wallet.to_str().unwrap()])
        .output()
        .expect("Failed to execute wallet init");
    assert!(wallet_output.status.success());
    assert!(tmp_wallet.exists());

    let wallet_data = fs::read_to_string(&tmp_wallet).expect("Read wallet failed");
    assert!(wallet_data.contains("public_key"));
    assert!(wallet_data.contains("secret_key"));

    let _ = fs::remove_file(&tmp_wallet);
    let _ = fs::remove_dir_all(&tmp_dir);
}
