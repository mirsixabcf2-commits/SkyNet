//! `agent_core_tests`: Unit tests for multicellular `AgentNode` organism growth,
//! `NanoSkn` internal conservation, `RestoreOrganismOptions`, and `SnowballEngine`.

use ed25519_dalek::SigningKey;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use skynet_cell::HardwareTier;
use skynet_ledger::{AgentMetabolicBank, NanoSkn, SknTransaction, TxOutput, UtxoDagLedger, UtxoId};

use crate::{AgentNode, RestoreOrganismOptions, SnowballEngine, SnowballParams};

#[test]
fn colossal_agent_grows_larger_than_tiny_peripheral_while_conserving_skn() {
    let mut rng = ChaCha8Rng::seed_from_u64(999);
    let (k1, k2) = (SigningKey::generate(&mut rng), SigningKey::generate(&mut rng));
    let alloc = NanoSkn::from_skn(10_000).unwrap();
    let (ledger, _) = UtxoDagLedger::from_genesis(vec![
        TxOutput::direct(k1.verifying_key().to_bytes(), alloc),
        TxOutput::direct(k2.verifying_key().to_bytes(), alloc),
    ]).unwrap();

    let mut giant = AgentNode::from_signing_key("Giant", 1, k1, HardwareTier::ColossalOrganism, ledger.clone(), SnowballParams::default()).unwrap();
    let mut tiny = AgentNode::from_signing_key("Tiny", 2, k2, HardwareTier::TinyPeripheral, ledger, SnowballParams::default()).unwrap();

    let (stim, target) = ([0.5, -0.3, 0.7, 0.2, -0.6, 0.4, 0.1, -0.8], [0.7, -0.6, 0.5, -0.4]);
    for _ in 0..12 {
        giant.run_biological_tick(&stim, &target).unwrap();
        tiny.run_biological_tick(&stim, &target).unwrap();
    }

    assert!(tiny.cells.len() <= 32);
    assert!(giant.cells.len() > tiny.cells.len() * 3);
    assert!(giant.total_apoptosis_events > 0);
    giant.verify_internal_skn_conservation().unwrap();
    tiny.verify_internal_skn_conservation().unwrap();
}

#[test]
fn restore_organism_and_snowball_finality_converge_cleanly() {
    let mut rng = ChaCha8Rng::seed_from_u64(4242);
    let key = SigningKey::generate(&mut rng);
    let alloc = NanoSkn::from_skn(500).unwrap();
    let (ledger, ghash) = UtxoDagLedger::from_genesis(vec![TxOutput::direct(key.verifying_key().to_bytes(), alloc)]).unwrap();

    let spawned = AgentNode::from_signing_key("Alpha", 42, key.clone(), HardwareTier::StandardNode, ledger.clone(), SnowballParams::default()).unwrap();
    let restored = AgentNode::restore_organism(RestoreOrganismOptions {
        name: "Alpha-Restored".into(), seed: 42, signing_key: key.clone(), tier: HardwareTier::StandardNode,
        ledger, cells: spawned.cells.clone(), bank: AgentMetabolicBank::new(spawned.bank.blood_pool),
        snowball_params: SnowballParams::default(),
    }).unwrap();
    restored.verify_internal_skn_conservation().unwrap();
    assert_eq!(restored.stats().total_cells, spawned.stats().total_cells);

    let utxo = UtxoId { tx_hash: ghash, vout: 0 };
    let tx_a = SknTransaction::new_signed(&key, vec![utxo], vec![TxOutput::direct([7; 32], alloc)], NanoSkn::ZERO, vec![ghash], 1);
    let tx_hash = tx_a.tx_hash();
    let mut sb = SnowballEngine::new(SnowballParams { sample_k: 4, quorum_alpha: 3, finality_beta: 3 });
    sb.register_tx(&tx_a);
    for round in 0..3 {
        for _ in 0..3 { sb.record_incoming_vote(utxo, tx_hash); }
        let finalized = sb.close_pulse_window();
        assert_eq!(finalized, if round < 2 { vec![] } else { vec![tx_hash] });
    }
    assert_eq!(sb.decided_for(&utxo), Some(tx_hash));
}
