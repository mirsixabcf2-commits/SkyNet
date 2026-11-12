//! `ledger_core_tests`: Unit tests for 21T SKN halving supply, UTXO-DAG conflict resolution, and Utreexo.

use ed25519_dalek::SigningKey;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{
    GenesisHalvingVault, HumanWallet, NanoSkn, SknTransaction, SynapticVoucher, TxOutput,
    UtxoDagLedger, UtxoId, VoucherChannelTracker, GENESIS_TOTAL_NANO_SKN,
};

#[test]
fn genesis_cap_and_annual_halving_preserve_exact_21_trillion_skn() {
    let seed = NanoSkn::from_skn(1_000_000_000_000).unwrap();
    let mut vault = GenesisHalvingVault::new(seed, 100).unwrap();

    assert_eq!(
        vault.annual_quota_for_year(0),
        NanoSkn::from_skn(10_000_000_000_000).unwrap()
    );
    assert_eq!(
        vault.annual_quota_for_year(1),
        NanoSkn::from_skn(5_000_000_000_000).unwrap()
    );
    assert_eq!(
        vault.annual_quota_for_year(2),
        NanoSkn::from_skn(2_500_000_000_000).unwrap()
    );

    for _ in 0..300 {
        vault.release_next_epoch().unwrap();
        assert_eq!(
            vault.total_accounted_supply().unwrap().0,
            GENESIS_TOTAL_NANO_SKN
        );
    }
}

fn build_direct_spend(
    signer: &SigningKey,
    utxo: UtxoId,
    recipient: ([u8; 32], u128),
    fee_and_nonce: (u128, u64),
) -> SknTransaction {
    SknTransaction::new_signed(
        signer,
        vec![utxo],
        vec![TxOutput::direct(recipient.0, NanoSkn::from_nano(recipient.1))],
        NanoSkn::from_nano(fee_and_nonce.0),
        vec![utxo.tx_hash],
        fee_and_nonce.1,
    )
}

#[test]
fn utxo_dag_detects_double_spend_and_conserves_nanoskn() {
    let mut rng = ChaCha8Rng::seed_from_u64(11);
    let alice = SigningKey::generate(&mut rng);
    let bob = SigningKey::generate(&mut rng);
    let carol = SigningKey::generate(&mut rng);

    let initial = NanoSkn::from_nano(5_000_000_001);
    let (mut ledger, gen_hash) = UtxoDagLedger::from_genesis(vec![TxOutput::direct(
        alice.verifying_key().to_bytes(),
        initial,
    )])
    .unwrap();
    let utxo0 = UtxoId { tx_hash: gen_hash, vout: 0 };

    let tx_bob = build_direct_spend(&alice, utxo0, (bob.verifying_key().to_bytes(), 5_000_000_000), (1, 1));
    let tx_carol = build_direct_spend(&alice, utxo0, (carol.verifying_key().to_bytes(), 4_999_999_990), (11, 2));

    let (h1, c1) = ledger.admit_pending_tx(tx_bob).unwrap();
    assert!(c1.is_empty());
    let (h2, c2) = ledger.admit_pending_tx(tx_carol).unwrap();
    assert_eq!(c2, vec![h1]);

    ledger.finalize_tx(&h1).unwrap();
    assert_eq!(
        ledger.balance_of(&bob.verifying_key().to_bytes()),
        NanoSkn::from_nano(5_000_000_000)
    );
    assert_eq!(ledger.total_tracked_supply().unwrap(), initial);
    assert!(ledger.finalize_tx(&h2).is_err());
}

fn build_500_bond_ledger(human_pk: [u8; 32], agent_pk: [u8; 32]) -> (UtxoDagLedger, [u8; 32]) {
    let outs = (0..500u128)
        .map(|i| {
            TxOutput::agent_bond(
                human_pk,
                agent_pk,
                NanoSkn::from_nano(100_000 + i),
                NanoSkn::from_nano(10_000),
            )
        })
        .collect();
    UtxoDagLedger::from_genesis(outs).unwrap()
}

fn stream_lossy_vouchers(
    tracker: &mut VoucherChannelTracker,
    payer: &SigningKey,
    target_id: UtxoId,
    payee_pk: [u8; 32],
) -> NanoSkn {
    let mut total_credited = NanoSkn::ZERO;
    for seq in 1..=100u64 {
        let v = SynapticVoucher::new_signed(
            payer,
            target_id,
            payee_pk,
            seq,
            NanoSkn::from_nano(seq as u128 * 5),
        );
        if seq % 25 != 0 {
            continue;
        }
        let delta = tracker.receive_udp_voucher(v).unwrap();
        total_credited = total_credited.checked_add(delta).unwrap();
    }
    total_credited
}

#[test]
fn utreexo_verifies_utxos_in_under_500_bytes_of_ram_and_vouchers_survive_udp_loss() {
    let human = HumanWallet::from_recovery_phrase("human-owner-1");
    let payee_pk = HumanWallet::from_recovery_phrase("provider-agent")
        .derive_agent_hotkey(0)
        .verifying_key()
        .to_bytes();
    let (payer_agent, _) = human.authorize_agent(0, NanoSkn::from_nano(10_000), 50);

    let (mut ledger, gen_hash) =
        build_500_bond_ledger(human.human_pubkey(), payer_agent.verifying_key().to_bytes());
    let forest = ledger.build_utreexo_forest();
    assert!(forest.roots.ram_footprint_bytes() < 500);

    let target_id = UtxoId { tx_hash: gen_hash, vout: 0 };
    let target_out = ledger.get_utxo(&target_id).unwrap().clone();
    let proof = forest.prove(&target_id).unwrap();
    forest.roots.verify_utxo(&target_id, &target_out, &proof).unwrap();

    let mut tracker = VoucherChannelTracker::new(target_id, NanoSkn::from_nano(10_000));
    let total_credited = stream_lossy_vouchers(&mut tracker, &payer_agent, target_id, payee_pk);
    assert_eq!(total_credited, NanoSkn::from_nano(500));

    let settle_tx = tracker
        .build_cooperative_settlement_tx(&payer_agent, &target_out, NanoSkn::ONE_NANO, vec![gen_hash], 1)
        .unwrap();
    let (shash, _) = ledger.admit_pending_tx(settle_tx).unwrap();
    ledger.finalize_tx(&shash).unwrap();
    assert_eq!(ledger.balance_of(&payee_pk), NanoSkn::from_nano(500));
}
