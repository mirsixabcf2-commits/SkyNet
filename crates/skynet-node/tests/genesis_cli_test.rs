//! Integration tests for `skynet genesis generate|verify`, real PoPE `airdrop claim`,
//! and real UDP `compute train` shard streaming (`T022`, `T043`).

use std::fs;
use std::net::UdpSocket;
use std::process::Command;
use std::time::Duration;

use skynet_node::load_genesis_file;
use skynet_wire::{ImpulseBody, ImpulsePacket, MicroTrainWireMessage, MAX_UDP_DATAGRAM_BYTES};

#[test]
fn test_genesis_cli_generate_verify_and_reject_single_nanoskn_tamper() {
    let bin = env!("CARGO_BIN_EXE_skynet");
    let dir = std::env::temp_dir().join(format!("skynet_gen_cli_{}", rand::random::<u64>()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let gen_path = dir.join("genesis.json");

    let gen_out = Command::new(bin)
        .args(["genesis", "generate", "--output", gen_path.to_str().unwrap(), "--network", "skynet-edge-v1"])
        .output()
        .expect("run genesis generate");
    assert!(gen_out.status.success(), "stderr: {}", String::from_utf8_lossy(&gen_out.stderr));
    let gen_stdout = String::from_utf8_lossy(&gen_out.stdout);
    assert!(gen_stdout.contains("GENESIS_HASH : 0x"));

    let ver_out = Command::new(bin)
        .args(["genesis", "verify", "--path", gen_path.to_str().unwrap()])
        .output()
        .expect("run genesis verify");
    assert!(ver_out.status.success());
    assert!(String::from_utf8_lossy(&ver_out.stdout).contains("PASSED"));

    let (mut cfg, _) = load_genesis_file(&gen_path).expect("load valid genesis");
    cfg.initial_utxos[0].amount.0 += 1;
    let tampered_path = dir.join("genesis_tampered.json");
    fs::write(&tampered_path, serde_json::to_string_pretty(&cfg).unwrap()).unwrap();

    let bad_out = Command::new(bin)
        .args(["genesis", "verify", "--path", tampered_path.to_str().unwrap()])
        .output()
        .expect("run genesis verify on tampered file");
    assert!(!bad_out.status.success(), "1-nanoSKN tamper must be rejected");

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_airdrop_pope_claim_and_duplicate_hwid_rejection() {
    let bin = env!("CARGO_BIN_EXE_skynet");
    let dir = std::env::temp_dir().join(format!("skynet_air_cli_{}", rand::random::<u64>()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let wallet_path = dir.join("wallet.json");
    let vault_path = dir.join("vault.bin");

    let w_out = Command::new(bin)
        .args(["wallet", "init", "--path", wallet_path.to_str().unwrap()])
        .output()
        .expect("wallet init");
    assert!(w_out.status.success());

    let c1 = Command::new(bin)
        .args(["airdrop", "claim", "--wallet", wallet_path.to_str().unwrap(), "--vault", vault_path.to_str().unwrap()])
        .output()
        .expect("first airdrop claim");
    assert!(c1.status.success(), "stderr: {}", String::from_utf8_lossy(&c1.stderr));
    assert!(String::from_utf8_lossy(&c1.stdout).contains("PoPE Hardware Attestation Verified"));

    let c2 = Command::new(bin)
        .args(["airdrop", "claim", "--wallet", wallet_path.to_str().unwrap(), "--vault", vault_path.to_str().unwrap()])
        .output()
        .expect("duplicate airdrop claim");
    assert!(!c2.status.success(), "duplicate HWID must be rejected by MarketingBootstrapVault");

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn test_compute_train_streams_manifest_and_fountain_droplets() {
    let bin = env!("CARGO_BIN_EXE_skynet");
    let dir = std::env::temp_dir().join(format!("skynet_train_cli_{}", rand::random::<u64>()));
    fs::create_dir_all(&dir).expect("create temp dir");

    let listener = UdpSocket::bind("127.0.0.1:0").expect("bind UDP listener");
    listener.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let target_addr = listener.local_addr().unwrap().to_string();

    let train_csv = dir.join("mini_dataset.csv");
    fs::write(&train_csv, "0.1,0.2,0.3,0.4\n0.5,0.6,0.7,0.8\n").unwrap();

    let t_out = Command::new(bin)
        .args(["compute", "train", "--input", train_csv.to_str().unwrap(), "--epochs", "1", "--bounty", "12", "--target", &target_addr])
        .output()
        .expect("compute train");
    assert!(t_out.status.success(), "stderr: {}", String::from_utf8_lossy(&t_out.stderr));

    let mut buf = [0u8; 1500];
    let (len1, _) = listener.recv_from(&mut buf).expect("receive manifest datagram");
    assert!(len1 <= MAX_UDP_DATAGRAM_BYTES);
    let pkt1 = ImpulsePacket::decode_and_verify(&buf[..len1]).expect("valid signed manifest");
    assert!(matches!(pkt1.body, ImpulseBody::MicroTrain(MicroTrainWireMessage::MicroBatchManifest(_))));

    let (len2, _) = listener.recv_from(&mut buf).expect("receive fountain droplet");
    assert!(len2 <= MAX_UDP_DATAGRAM_BYTES);
    let pkt2 = ImpulsePacket::decode_and_verify(&buf[..len2]).expect("valid signed droplet");
    assert!(matches!(pkt2.body, ImpulseBody::FountainDroplet(_)));

    let _ = fs::remove_dir_all(dir);
}
