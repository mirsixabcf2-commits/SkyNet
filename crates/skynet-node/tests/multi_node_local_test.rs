//! Integration test verifying parallel execution of multiple local nodes with independent storage.

use std::net::SocketAddr;
use skynet_cell::HardwareTier;
use skynet_node::{NodeConfig, NodeRuntime, StorageEngine};
use skynet_wire::{ImpulseBody, ImpulsePacket};

#[tokio::test]
async fn test_two_nodes_run_parallel_without_storage_conflict() {
    let tmp_dir1 = std::env::temp_dir().join(format!("skynet_node1_{}", rand::random::<u64>()));
    let tmp_dir2 = std::env::temp_dir().join(format!("skynet_node2_{}", rand::random::<u64>()));

    let port1 = 19891;
    let port2 = 19892;

    let cfg1 = NodeConfig {
        port: port1,
        data_dir: tmp_dir1.clone(),
        name: "Node-Alpha".to_string(),
        tier: HardwareTier::StandardNode,
        seed_peers: vec![SocketAddr::from(([127, 0, 0, 1], port2))],
        owner_wallet: None,
        tick_interval_ms: 1000,
        auto_flush_interval_sec: 30,
    };

    let cfg2 = NodeConfig {
        port: port2,
        data_dir: tmp_dir2.clone(),
        name: "Node-Beta".to_string(),
        tier: HardwareTier::TinyPeripheral,
        seed_peers: vec![SocketAddr::from(([127, 0, 0, 1], port1))],
        owner_wallet: None,
        tick_interval_ms: 1000,
        auto_flush_interval_sec: 30,
    };

    let runtime1 = NodeRuntime::init(cfg1).await.expect("node1 init must succeed");
    let mut runtime2 = NodeRuntime::init(cfg2).await.expect("node2 init must succeed");

    assert_ne!(runtime1.agent.pubkey, runtime2.agent.pubkey);

    // Send UDP packet from node1 to node2
    let pkt = ImpulsePacket::new_signed(
        &runtime1.agent.signing_key,
        0,
        0,
        1,
        0,
        1,
        ImpulseBody::Heartbeat {
            tier_code: 2,
            total_cells: runtime1.agent.cells.len() as u32,
            stem_cells: 2,
            on_chain_nano_skn: runtime1.agent.ledger.balance_of(&runtime1.agent.pubkey).0,
            elite_dna_hash: [0u8; 32],
        },
    )
    .expect("packet creation must succeed");

    let datagram = pkt.encode_datagram().expect("encode must succeed");
    let target = SocketAddr::from(([127, 0, 0, 1], port2));
    runtime1.socket.send_to(&datagram, target).await.expect("send must succeed");

    let mut buf = [0u8; 2048];
    let (len, from_addr) = runtime2.socket.recv_from(&mut buf).await.expect("recv must succeed");
    assert_eq!(from_addr.port(), port1);

    let res = runtime2.agent.handle_incoming_datagram(&buf[..len]);
    assert!(res.is_ok());

    runtime1.storage.save_all(&runtime1.agent).unwrap();
    runtime2.storage.save_all(&runtime2.agent).unwrap();

    let storage1 = StorageEngine::new(&tmp_dir1).unwrap();
    let storage2 = StorageEngine::new(&tmp_dir2).unwrap();

    let meta1 = storage1.load_metadata().unwrap().unwrap();
    let meta2 = storage2.load_metadata().unwrap().unwrap();

    assert_eq!(meta1.name, "Node-Alpha");
    assert_eq!(meta2.name, "Node-Beta");
    assert_ne!(meta1.pubkey, meta2.pubkey);

    let _ = std::fs::remove_dir_all(tmp_dir1);
    let _ = std::fs::remove_dir_all(tmp_dir2);
}
