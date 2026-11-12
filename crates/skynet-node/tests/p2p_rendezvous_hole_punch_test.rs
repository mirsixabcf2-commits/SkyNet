//! Integration tests for 1 Seed + 2 NAT Worker P2P `PheromoneHello` -> `PunchRendezvous` ->
//! `< 3s` UDP hole punching, `< 1s` `LanBeacon`, `GangliaRelay`, and `EpidermalFilter` (`T022`).

use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::time::{Duration, Instant};

use skynet_cell::HardwareTier;
use skynet_ledger::default_mainnet_genesis;
use skynet_node::runtime_p2p::unix_now_sec;
use skynet_node::{write_genesis_file, NodeConfig, NodeRuntime};
use skynet_wire::{
    EpidermalFilter, EpidermalHeader, GangliaRelayFrame, ImpulseBody, ImpulsePacket,
    RendezvousMessage,
};

async fn spawn_node(name: &str, genesis_path: &Path, seeds: Vec<SocketAddr>) -> NodeRuntime {
    let dir = std::env::temp_dir().join(format!("skynet_p2p_{name}_{}", rand::random::<u64>()));
    let cfg = NodeConfig {
        port: 0,
        data_dir: dir,
        name: name.to_string(),
        tier: HardwareTier::StandardNode,
        seed_peers: seeds,
        owner_wallet: None,
        genesis_path: Some(genesis_path.to_path_buf()),
        tick_interval_ms: 1_000,
        auto_flush_interval_sec: 60,
    };
    NodeRuntime::init(cfg).await.expect("node init must succeed")
}

async fn pump_once(node: &mut NodeRuntime) {
    let mut buf = [0u8; 2048];
    while let Ok(Ok((len, src))) =
        tokio::time::timeout(Duration::from_millis(4), node.socket.recv_from(&mut buf)).await
    {
        let _ = node
            .p2p
            .process_datagram(&node.socket, &mut node.agent, (&buf[..len], src))
            .await
            .expect("process datagram");
    }
}

#[tokio::test]
async fn test_seed_nat_hole_punch_under_3s_and_ganglia_relay() {
    let dir = std::env::temp_dir().join(format!("skynet_p2p_suite_{}", rand::random::<u64>()));
    let genesis_path = dir.join("genesis.json");
    let _ghash = write_genesis_file(&genesis_path, &default_mainnet_genesis()).unwrap();

    let mut seed = spawn_node("Seed", &genesis_path, vec![]).await;
    let seed_addr = SocketAddr::from(([127, 0, 0, 1], seed.config.port));
    let seed_pk = seed.agent.pubkey;

    let mut nat_a = spawn_node("NAT-A", &genesis_path, vec![seed_addr]).await;
    let mut nat_b = spawn_node("NAT-B", &genesis_path, vec![seed_addr]).await;
    nat_a.agent.rendezvous.add_trusted_seed(seed_pk);
    nat_b.agent.rendezvous.add_trusted_seed(seed_pk);
    let (pk_a, pk_b) = (nat_a.agent.pubkey, nat_b.agent.pubkey);

    let start = Instant::now();
    nat_a.p2p.send_seed_keepalives(&nat_a.socket, &mut nat_a.agent, &[seed_addr]).await.unwrap();
    nat_b.p2p.send_seed_keepalives(&nat_b.socket, &mut nat_b.agent, &[seed_addr]).await.unwrap();

    for _ in 0..30 {
        pump_once(&mut seed).await;
        pump_once(&mut nat_a).await;
        pump_once(&mut nat_b).await;
        let a_ok = nat_a.agent.rendezvous.get_peer(&pk_b).is_some_and(|p| p.direct_reachable);
        let b_ok = nat_b.agent.rendezvous.get_peer(&pk_a).is_some_and(|p| p.direct_reachable);
        if a_ok && b_ok {
            break;
        }
    }
    assert!(start.elapsed() < Duration::from_secs(3), "NAT hole punch exceeded 3s SLA");
    assert!(nat_a.agent.rendezvous.get_peer(&pk_b).is_some_and(|p| p.direct_reachable));
    assert!(nat_b.agent.rendezvous.get_peer(&pk_a).is_some_and(|p| p.direct_reachable));

    verify_ganglia_relay(&nat_a, &mut seed, &mut nat_b, seed_addr).await;

    for d in [&seed.config.data_dir, &nat_a.config.data_dir, &nat_b.config.data_dir, &dir] {
        let _ = fs::remove_dir_all(d);
    }
}

async fn verify_ganglia_relay(
    nat_a: &NodeRuntime,
    seed: &mut NodeRuntime,
    nat_b: &mut NodeRuntime,
    seed_addr: SocketAddr,
) {
    let before_b = nat_b.agent.udp_received;
    let inner_hb = ImpulsePacket::new_signed(
        &nat_a.agent.signing_key, 0, 0, 2, 0, 99,
        ImpulseBody::Heartbeat { tier_code: 2, total_cells: 0, stem_cells: 0, on_chain_nano_skn: 0, elite_dna_hash: [0; 32] },
    ).unwrap();
    let relay = GangliaRelayFrame::new(nat_b.agent.pubkey, nat_a.agent.pubkey, inner_hb.encode_datagram().unwrap()).unwrap();
    let outer = ImpulsePacket::new_signed(
        &nat_a.agent.signing_key, 0, 0, 4, 0, 100,
        ImpulseBody::Rendezvous(RendezvousMessage::GangliaRelayFrame(relay)),
    ).unwrap();
    let wire = nat_a.p2p.encode_wire_packet(&outer, unix_now_sec()).unwrap();
    nat_a.socket.send_to(&wire, seed_addr).await.unwrap();

    pump_once(seed).await;
    pump_once(nat_b).await;
    assert!(nat_b.agent.udp_received > before_b, "GangliaRelay inner packet must reach NAT-B");
}

#[tokio::test]
async fn test_lan_beacon_under_1s_and_wrong_genesis_hash_rejection() {
    let dir = std::env::temp_dir().join(format!("skynet_lan_suite_{}", rand::random::<u64>()));
    let genesis_path = dir.join("genesis.json");
    let ghash = write_genesis_file(&genesis_path, &default_mainnet_genesis()).unwrap();

    let mut node_a = spawn_node("LAN-A", &genesis_path, vec![]).await;
    let mut node_b = spawn_node("LAN-B", &genesis_path, vec![]).await;
    let addr_b = SocketAddr::from(([127, 0, 0, 1], node_b.config.port));

    let lan_start = Instant::now();
    node_a.p2p.add_lan_target(addr_b);
    node_a.p2p.broadcast_lan_beacon(&node_a.socket, &mut node_a.agent).await.unwrap();
    pump_once(&mut node_b).await;

    assert!(lan_start.elapsed() < Duration::from_secs(1), "LAN discovery must finish < 1s");
    assert!(node_b.agent.rendezvous.get_peer(&node_a.agent.pubkey).is_some_and(|p| p.direct_reachable));

    // Foreign GENESIS_HASH and skewed timestamp (> 30s) must be dropped by EpidermalFilter
    let rx_before = node_b.agent.udp_received;
    let hb = ImpulsePacket::new_signed(
        &node_a.agent.signing_key, 0, 0, 2, 0, 5,
        ImpulseBody::Heartbeat { tier_code: 2, total_cells: 0, stem_cells: 0, on_chain_nano_skn: 0, elite_dna_hash: [0; 32] },
    ).unwrap();
    let bad_net = EpidermalFilter::encode_framed_packet(&hb, &[0xEE; 32], unix_now_sec()).unwrap();
    let stale_hdr = EpidermalHeader::new(ghash, unix_now_sec() + 90, 2);
    let stale_pkt = EpidermalFilter::encode_framed(&stale_hdr, &hb.encode_datagram().unwrap()).unwrap();

    assert!(!node_b.p2p.process_datagram(&node_b.socket, &mut node_b.agent, (&bad_net, addr_b)).await.unwrap());
    assert!(!node_b.p2p.process_datagram(&node_b.socket, &mut node_b.agent, (&stale_pkt, addr_b)).await.unwrap());
    assert_eq!(node_b.agent.udp_received, rx_before);

    for d in [&node_a.config.data_dir, &node_b.config.data_dir, &dir] {
        let _ = fs::remove_dir_all(d);
    }
}
