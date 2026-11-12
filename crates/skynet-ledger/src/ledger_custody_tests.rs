//! `ledger_custody_tests`: Unit tests for Soul-Body Custody covenants and unilateral voucher settlement.

use ed25519_dalek::SigningKey;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{
    AgentFundingParams, HumanWallet, LedgerError, NanoSkn, SknTransaction, SynapticVoucher,
    TxOutput, UtxoDagLedger, UtxoId, VoucherChannelTracker,
};

fn setup_funded_agent_bond(
    human: &HumanWallet,
    initial_skn: NanoSkn,
    bond_skn: NanoSkn,
    allowance: NanoSkn,
) -> (UtxoDagLedger, SigningKey, [u8; 32], UtxoId) {
    let (mut ledger, gen_hash) =
        UtxoDagLedger::from_genesis(vec![TxOutput::direct(human.human_pubkey(), initial_skn)])
            .unwrap();
    let (agent_key, cert) = human.authorize_agent(0, allowance, 100);
    assert!(cert.verify(5).is_ok());

    let fund_tx = human
        .create_agent_funding_tx(AgentFundingParams {
            cold_utxo_id: UtxoId { tx_hash: gen_hash, vout: 0 },
            cold_utxo_amount: initial_skn,
            agent_pubkey: cert.agent_pubkey,
            bond_amount: bond_skn,
            max_voucher_allowance: allowance,
            fee: NanoSkn::ONE_NANO,
            dag_parents: vec![gen_hash],
            nonce: 1,
        })
        .unwrap();
    let (fund_hash, _) = ledger.admit_pending_tx(fund_tx).unwrap();
    let minted = ledger.finalize_tx(&fund_hash).unwrap();
    (ledger, agent_key, fund_hash, minted[0])
}

#[test]
fn human_wallet_protects_principal_against_compromised_agent_and_supports_emergency_reclaim() {
    let human = HumanWallet::from_recovery_phrase("pain skynet sovereign human seed 2026");
    let thief = HumanWallet::from_recovery_phrase("evil attacker wallet");
    let (initial_skn, bond_skn, allowance) = (
        NanoSkn::from_skn(1_000).unwrap(),
        NanoSkn::from_skn(400).unwrap(),
        NanoSkn::from_skn(10).unwrap(),
    );
    let (mut ledger, agent_key, fund_hash, bond_id) =
        setup_funded_agent_bond(&human, initial_skn, bond_skn, allowance);
    assert_eq!(ledger.human_total_net_worth(&human.human_pubkey()), initial_skn.checked_sub(NanoSkn::ONE_NANO).unwrap());

    let rem = NanoSkn::from_skn(50).unwrap().checked_sub(NanoSkn::ONE_NANO).unwrap();
    let outs = vec![
        TxOutput::direct(thief.human_pubkey(), NanoSkn::from_skn(350).unwrap()),
        TxOutput::direct(human.human_pubkey(), rem),
    ];
    let hack_tx = SknTransaction::new_signed(&agent_key, vec![bond_id], outs, NanoSkn::ONE_NANO, vec![fund_hash], 2);
    assert_eq!(ledger.admit_pending_tx(hack_tx), Err(LedgerError::HumanPrincipalDiversionBlocked));

    let reclaim_tx = human.build_sovereign_reclaim_tx(bond_id, bond_skn, NanoSkn::ONE_NANO, vec![fund_hash], 3).unwrap();
    let (rec_hash, _) = ledger.admit_pending_tx(reclaim_tx).unwrap();
    ledger.finalize_tx(&rec_hash).unwrap();
    assert_eq!(ledger.balance_of(&agent_key.verifying_key().to_bytes()), NanoSkn::ZERO);
    assert_eq!(ledger.balance_of(&human.human_pubkey()), initial_skn.checked_sub(NanoSkn::from_nano(2)).unwrap());
}

#[test]
fn test_unilateral_voucher_settlement_without_payer_cooperation() {
    let mut rng = ChaCha8Rng::seed_from_u64(8888);
    let human = HumanWallet::from_master_seed([0x77u8; 32]);
    let payer_agent = SigningKey::generate(&mut rng);
    let payee_agent = SigningKey::generate(&mut rng);
    let payee_pk = payee_agent.verifying_key().to_bytes();

    let (initial_bond, allowance) = (NanoSkn::from_nano(50_000), NanoSkn::from_nano(5_000));
    let bond_out = TxOutput::agent_bond(human.human_pubkey(), payer_agent.verifying_key().to_bytes(), initial_bond, allowance);
    let (mut ledger, gen_hash) = UtxoDagLedger::from_genesis(vec![bond_out.clone()]).unwrap();
    let bond_utxo = UtxoId { tx_hash: gen_hash, vout: 0 };

    let mut tracker = VoucherChannelTracker::new(bond_utxo, allowance);
    let voucher = SynapticVoucher::new_signed(&payer_agent, bond_utxo, payee_pk, 1, NanoSkn::from_nano(2_500));
    tracker.receive_udp_voucher(voucher).unwrap();

    let (tx, highest_v) = tracker.build_unilateral_settlement_tx(&payee_agent, &bond_out, NanoSkn::ONE_NANO, vec![gen_hash], 1).unwrap();
    let (tx_hash, _) = ledger.admit_unilateral_voucher_settlement(tx, &highest_v).unwrap();
    assert_eq!(ledger.finalize_tx(&tx_hash).unwrap().len(), 2);
    assert_eq!(ledger.balance_of(&payee_pk), NanoSkn::from_nano(2_500));
    assert_eq!(ledger.human_total_net_worth(&human.human_pubkey()), initial_bond.checked_sub(NanoSkn::from_nano(2_501)).unwrap());
}
