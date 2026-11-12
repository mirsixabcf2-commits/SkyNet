//! `agent_network_tests`: Unit tests for UDP `datagram_router` security gates,
//! Snowball unsolicited-reply rejection, cluster mitosis quorum, and live Compute/Rendezvous/MicroTrain wiring.

use ed25519_dalek::SigningKey;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use skynet_cell::compute_candle::CandleTaskInput;
use skynet_cell::HardwareTier;
use skynet_ledger::{NanoSkn, TxOutput, UtxoDagLedger, UtxoId};
use skynet_wire::{
    ComputeMessage, ComputeTaskManifest, ImpulseBody, ImpulsePacket, LanBeacon, PeerContact,
    RendezvousMessage,
};

use crate::{AgentNode, SnowballParams};

fn spawn_test_node(seed: u64, key: SigningKey, skn: u128) -> AgentNode {
    let alloc = NanoSkn::from_skn(skn).unwrap();
    let (ledger, _) = UtxoDagLedger::from_genesis(vec![TxOutput::direct(key.verifying_key().to_bytes(), alloc)]).unwrap();
    AgentNode::from_signing_key("Node", seed, key, HardwareTier::StandardNode, ledger, SnowballParams::default()).unwrap()
}

#[test]
fn test_unsolicited_snowball_reply_is_dropped() {
    let mut rng = ChaCha8Rng::seed_from_u64(1234);
    let (node_key, attacker_key) = (SigningKey::generate(&mut rng), SigningKey::generate(&mut rng));
    let mut node = spawn_test_node(1, node_key, 100);

    let fake_utxo = UtxoId { tx_hash: [1u8; 32], vout: 0 };
    let body = ImpulseBody::SnowballReply { query_id: 999, votes: vec![(fake_utxo, [2u8; 32])] };
    let reply = ImpulsePacket::new_signed(&attacker_key, node.cluster_prefix, node.prefix_len, 2, node.epoch, 1, body).unwrap();

    node.handle_incoming_datagram(&reply.encode_datagram().unwrap()).unwrap();
    assert!(node.snowball.vote_for(&fake_utxo).is_none());
}

#[test]
fn test_cluster_mitosis_requires_peer_quorum() {
    let mut rng = ChaCha8Rng::seed_from_u64(5678);
    let (node_key, stranger_key, peer_key) = (SigningKey::generate(&mut rng), SigningKey::generate(&mut rng), SigningKey::generate(&mut rng));
    let mut node = spawn_test_node(1, node_key, 100);
    let init_len = node.prefix_len;
    let body = ImpulseBody::ClusterMitosis { parent_prefix: node.cluster_prefix, parent_prefix_len: node.prefix_len, epoch: node.epoch };

    let stranger_pkt = ImpulsePacket::new_signed(&stranger_key, node.cluster_prefix, node.prefix_len, 2, node.epoch, 1, body.clone()).unwrap();
    node.handle_incoming_datagram(&stranger_pkt.encode_datagram().unwrap()).unwrap();
    assert_eq!(node.prefix_len, init_len);

    node.peer_table.observe_peer(PeerContact {
        pubkey: peer_key.verifying_key().to_bytes(), addr: "10.0.0.1:8888".parse().unwrap(),
        cluster_prefix: node.cluster_prefix, prefix_len: node.prefix_len, bonded_skn_whole: 100, synaptic_score: 200,
    });
    let peer_pkt1 = ImpulsePacket::new_signed(&peer_key, node.cluster_prefix, node.prefix_len, 2, node.epoch, 1, body).unwrap();
    node.handle_incoming_datagram(&peer_pkt1.encode_datagram().unwrap()).unwrap();
    assert_eq!(node.prefix_len, init_len);
}

#[test]
fn datagram_router_dispatches_live_compute_and_rendezvous_impulses() {
    let mut rng = ChaCha8Rng::seed_from_u64(9001);
    let (node_k, client_k) = (SigningKey::generate(&mut rng), SigningKey::generate(&mut rng));
    let mut node = spawn_test_node(3, node_k, 500);

    let beacon = ImpulsePacket::new_signed(&client_k, 0, 0, 2, 0, 1, ImpulseBody::Rendezvous(RendezvousMessage::LanBeacon(LanBeacon {
        sender_pubkey: client_k.verifying_key().to_bytes(), listen_port: 9000, genesis_hash: [5; 32],
    }))).unwrap();
    node.handle_incoming_datagram_from(&beacon.encode_datagram().unwrap(), Some("192.168.1.44:9000".parse().unwrap())).unwrap();
    assert_eq!(node.rendezvous.peer_count(), 1);

    let announce = ImpulsePacket::new_signed(&client_k, 0, 0, 2, 0, 2, ImpulseBody::Compute(ComputeMessage::TaskAnnouncement(ComputeTaskManifest {
        task_id: [0xAB; 32], client_pubkey: client_k.verifying_key().to_bytes(), opcode: 0x1101, required_tier: 2,
        input_hash: [0; 32], timeout_epoch: 50, max_ticks: 1000, bounty_nano_skn: 50_000, min_worker_bond_nano_skn: 5_000,
        auditor_bounty_nano_skn: 0, required_auditors: 0,
    }))).unwrap();
    let bid_reply = node.handle_incoming_datagram(&announce.encode_datagram().unwrap()).unwrap().expect("expected real ComputeBid reply");
    assert!(matches!(bid_reply.body, ImpulseBody::Compute(ComputeMessage::TaskBid(_))));

    let input = CandleTaskInput { task_id: [0xCD; 32], dim_in: 2, dim_hidden: 3, dim_out: 1, weights: vec![0.05; 13], input: vec![0.4, -0.2] };
    let direct = ImpulsePacket::new_signed(&client_k, 0, 0, 2, 0, 3, ImpulseBody::Compute(ComputeMessage::DirectPayload {
        task_id: [0xCD; 32], is_result: false, data: bincode::serialize(&input).unwrap(),
    })).unwrap();
    let claim_reply = node.handle_incoming_datagram(&direct.encode_datagram().unwrap()).unwrap().expect("expected sub-10ms CompletionClaim");
    assert!(matches!(claim_reply.body, ImpulseBody::Compute(ComputeMessage::CompletionClaim(_))));
}
