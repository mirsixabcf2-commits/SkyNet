//! `wire_tests`: Core unit tests for `ImpulsePacket`, GF(2) Fountain recovery,
//! and `SynapticPeerTable` memory/Eclipse invariants.

use std::net::SocketAddr;

use ed25519_dalek::SigningKey;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use skynet_dna::CellDna;

use crate::{
    FountainDecoder, FountainEncoder, ImpulseBody, ImpulsePacket, PeerContact, SynapticPeerTable,
    WireError, MAX_UDP_DATAGRAM_BYTES,
};

fn absorb_droplet_wire(
    wire_bytes: &[u8],
    decoder: &mut Option<FountainDecoder>,
) -> Option<Vec<u8>> {
    let decoded_pkt = ImpulsePacket::decode_and_verify(wire_bytes).unwrap();
    let ImpulseBody::FountainDroplet(d) = decoded_pkt.body else {
        return None;
    };
    let dec = decoder.get_or_insert_with(|| FountainDecoder::from_first_droplet(&d));
    dec.absorb(d).unwrap()
}

#[test]
fn fountain_codes_reconstruct_cell_dna_over_40_percent_udp_packet_loss() {
    let dna = CellDna::genesis_stem(2026);
    let raw_dna = bincode::serialize(&dna).unwrap();
    assert!(raw_dna.len() > MAX_UDP_DATAGRAM_BYTES);

    let key = SigningKey::from_bytes(&[7u8; 32]);
    let mut enc = FountainEncoder::new(&raw_dna).unwrap();
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let mut decoder: Option<FountainDecoder> = None;
    let mut recovered: Option<Vec<u8>> = None;
    let mut sent = 0usize;
    let mut survived = 0usize;

    while recovered.is_none() && sent < 100 {
        let droplet = enc.next_droplet();
        let pkt = ImpulsePacket::new_signed(
            &key,
            0,
            0,
            4,
            1,
            sent as u32,
            ImpulseBody::FountainDroplet(droplet),
        )
        .unwrap();
        let wire_bytes = pkt.encode_datagram().unwrap();
        assert!(wire_bytes.len() <= MAX_UDP_DATAGRAM_BYTES);
        sent += 1;
        if rng.gen_bool(0.40) {
            continue;
        }
        survived += 1;
        recovered = absorb_droplet_wire(&wire_bytes, &mut decoder);
    }

    assert!(survived < sent);
    let reconstructed_dna: CellDna = bincode::deserialize(&recovered.unwrap()).unwrap();
    assert_eq!(reconstructed_dna.compute_hash(), dna.compute_hash());
}

#[test]
fn tampered_udp_datagram_is_rejected() {
    let key = SigningKey::from_bytes(&[9u8; 32]);
    let pkt = ImpulsePacket::new_signed(
        &key,
        0b10,
        2,
        3,
        10,
        1,
        ImpulseBody::Heartbeat {
            tier_code: 2,
            total_cells: 500,
            stem_cells: 40,
            on_chain_nano_skn: 1_000_000_000,
            elite_dna_hash: [1u8; 32],
        },
    )
    .unwrap();
    let mut bytes = pkt.encode_datagram().unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF;
    assert!(ImpulsePacket::decode_and_verify(&bytes).is_err());
}

#[test]
fn invalid_hop_ttl_rejected_before_crypto_eval() {
    let key = SigningKey::from_bytes(&[9u8; 32]);
    let pkt = ImpulsePacket::new_signed(
        &key,
        0b10,
        2,
        0,
        10,
        1,
        ImpulseBody::Heartbeat {
            tier_code: 2,
            total_cells: 10,
            stem_cells: 2,
            on_chain_nano_skn: 100,
            elite_dna_hash: [0u8; 32],
        },
    )
    .unwrap();
    let bytes = pkt.encode_datagram().unwrap();
    let err = ImpulsePacket::decode_and_verify(&bytes).unwrap_err();
    assert!(matches!(err, WireError::InvalidHopTtl { .. }));
}

#[test]
fn synaptic_peer_table_bounds_10000_ips_under_6kb_ram_with_logarithmic_buckets() {
    let local_pub = [0xAAu8; 32];
    let mut table = SynapticPeerTable::new(&local_pub, 0b01, 2, 12, 2);
    let mut rng = ChaCha8Rng::seed_from_u64(9999);

    for i in 0..10_000u32 {
        if i % 10 == 0 {
            table.decay_synapses();
        }
        let mut pk = [0u8; 32];
        rng.fill(&mut pk);
        let port = (10_000 + (i % 50_000)) as u16;
        let addr: SocketAddr = format!(
            "10.{}.{}.{}:{}",
            (i >> 16) & 255,
            (i >> 8) & 255,
            i & 255,
            port
        )
        .parse()
        .unwrap();
        table.observe_peer(PeerContact {
            pubkey: pk,
            addr,
            cluster_prefix: i % 4,
            prefix_len: 2,
            bonded_skn_whole: (i as u64 % 500) * 10,
            synaptic_score: 0,
        });
    }

    assert_eq!(table.total_observed_ips_ever, 10_000);
    assert!(table.stored_peer_count() <= 64);
    assert!(table.estimated_ram_bytes() < 6_000);
    assert!(table.total_hebbian_evictions > 1_000);
}

#[test]
fn test_hebbian_resists_eclipse_decay_attack() {
    let local_pub = [0x55u8; 32];
    let mut table = SynapticPeerTable::new(&local_pub, 0, 0, 4, 1);
    let strong_pub = [0x11u8; 32];
    table.observe_peer(PeerContact {
        pubkey: strong_pub,
        addr: "192.168.1.10:8888".parse().unwrap(),
        cluster_prefix: 0,
        prefix_len: 0,
        bonded_skn_whole: 1000,
        synaptic_score: 500,
    });
    let initial_score = table.intra_cluster_peers[0].synaptic_score;

    for i in 0..50 {
        let mut fake_pk = [0u8; 32];
        fake_pk[0] = (i + 1) as u8;
        table.observe_peer(PeerContact {
            pubkey: fake_pk,
            addr: format!("10.0.0.{}:8888", i + 1).parse().unwrap(),
            cluster_prefix: 0,
            prefix_len: 0,
            bonded_skn_whole: 0,
            synaptic_score: 1,
        });
    }

    let incumbent = table
        .intra_cluster_peers
        .iter()
        .find(|p| p.pubkey == strong_pub)
        .unwrap();
    assert_eq!(incumbent.synaptic_score, initial_score);
}
