//! End-to-end Opportunistic Edge Micro-Training, Canary Fraud Slashing,
//! Unilateral `SynapticVoucher` Settlement (slice 6 disconnect), and `Yield Sweep` test (`T041`).

use ed25519_dalek::SigningKey;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use skynet_agent::{
    AgentNode, OpportunisticWorkerEngine, ShardExecutionStep, SnowballParams,
    SystemTelemetrySnapshot,
};
use skynet_cell::{
    compute_canonical_digest, decode_dense_f16, encode_dense_f16, execute_micro_train_slices,
    generate_canary_batch, HardwareTier, MicroTrainConfig, MicroTrainOutcome, PreemptGuard,
};
use skynet_ledger::compute_escrow::ComputeEscrowVault;
use skynet_ledger::{
    validate_genesis_invariants, CustodyPolicy, GenesisConfig, GenesisHalvingVault,
    GenesisUtxoEntry, HumanWallet, NanoSkn, SynapticVoucher, TxOutput, UtxoDagLedger, UtxoId,
    YieldSweepConfig, DEFAULT_MAINNET_NETWORK_NAME, GENESIS_CIRCULATING_NANO_SKN,
    GENESIS_TOTAL_NANO_SKN, GENESIS_VAULT_NANO_SKN,
};
use skynet_wire::{
    encode_micro_train_shard, ChunkVoucherDelivery, ImpulseBody, ImpulsePacket, MicroBatchManifest,
    MicroTrainWireMessage, WeightDeltaClaim, MAX_UDP_DATAGRAM_BYTES,
};

fn build_shard_manifest_and_bytes(
    ids: ([u8; 32], [u8; 32], u16),
    cfg: &MicroTrainConfig,
) -> (MicroBatchManifest, Vec<u8>) {
    let (task_id, client_pk, shard_id) = ids;
    let mut batch = generate_canary_batch(&[(shard_id as u8).wrapping_add(1); 32], 1, shard_id, cfg);
    batch.task_id = task_id;
    let raw = bincode::serialize(&batch).expect("serialize MicroTrainBatch");
    let shard_object_hash = *blake3::hash(&raw).as_bytes();
    let manifest = MicroBatchManifest {
        task_id, client_pubkey: client_pk, opcode: 0x3100, shard_id, total_shards: 10,
        dim_in: cfg.dim_in as u16, dim_hidden: cfg.dim_hidden as u16, dim_out: cfg.dim_out as u16,
        inner_steps: cfg.inner_steps, learning_rate_micro: cfg.learning_rate_micro,
        shard_object_hash, shard_bytes_len: raw.len() as u32,
        bounty_per_chunk_nano: 10_000, min_worker_bond_nano: 1_000,
    };
    (manifest, raw)
}

fn stream_shard_to_worker(
    worker: &mut AgentNode,
    client_key: &SigningKey,
    manifest: MicroBatchManifest,
    shard_bytes: &[u8],
) -> WeightDeltaClaim {
    let m_pkt = ImpulsePacket::new_signed(client_key, 0, 0, 4, 1, 1, ImpulseBody::MicroTrain(MicroTrainWireMessage::MicroBatchManifest(manifest))).unwrap();
    worker.handle_incoming_datagram(&m_pkt.encode_datagram().unwrap()).unwrap();

    let mut enc = encode_micro_train_shard(shard_bytes).unwrap();
    let mut rng = ChaCha8Rng::seed_from_u64(777);
    for seq in 0..80u32 {
        let d_pkt = ImpulsePacket::new_signed(client_key, 0, 0, 4, 1, seq + 2, ImpulseBody::FountainDroplet(enc.next_droplet())).unwrap();
        let wire = d_pkt.encode_datagram().unwrap();
        assert!(wire.len() <= MAX_UDP_DATAGRAM_BYTES);
        if rng.gen_bool(0.35) { continue; }
        if let Some(reply) = worker.handle_incoming_datagram(&wire).unwrap() {
            assert!(reply.encode_datagram().unwrap().len() <= MAX_UDP_DATAGRAM_BYTES);
            if let ImpulseBody::MicroTrain(MicroTrainWireMessage::WeightDeltaClaim(claim)) = reply.body { return claim; }
        }
    }
    panic!("expected WeightDeltaClaim after Fountain recovery under 35% packet loss");
}

fn assert_canary_catches_dishonest_worker(auditor_node: &mut AgentNode, client_k: &SigningKey, cheat_k: &SigningKey, cfg: &MicroTrainConfig) {
    let canary_batch = generate_canary_batch(&[0x5A; 32], 1, 0, cfg);
    let canary_task = canary_batch.task_id;
    auditor_node.compute_coordinator.escrow_vault.lock_task_simple(canary_task, client_k.verifying_key().to_bytes(), cheat_k.verifying_key().to_bytes(), NanoSkn(50_000), NanoSkn(30_000), 100).unwrap();
    let MicroTrainOutcome::Completed(ref_res) = execute_micro_train_slices(cfg, &canary_batch, None, &PreemptGuard::new()).unwrap() else { panic!("ref"); };
    auditor_node.opportunistic_worker.register_canary_reference(ref_res.clone());

    let flipped: Vec<f32> = ref_res.decode_delta_f32().into_iter().map(|v| -v).collect();
    let bad_f16 = encode_dense_f16(&flipped);
    let bad_claim = WeightDeltaClaim {
        task_id: canary_task, shard_id: 0, worker_pubkey: [0; 32], completed_steps: cfg.inner_steps,
        initial_loss_micro: 800_000, final_loss_micro: 400_000, canonical_digest: compute_canonical_digest(&decode_dense_f16(&bad_f16)),
        inline_delta_f16: bad_f16, delta_object_hash: None, worker_signature: Vec::new(),
    }.into_signed(cheat_k).unwrap();
    let cheat_pkt = ImpulsePacket::new_signed(cheat_k, 0, 0, 4, 1, 1, ImpulseBody::MicroTrain(MicroTrainWireMessage::WeightDeltaClaim(bad_claim))).unwrap();

    let fraud_reply = auditor_node.handle_incoming_datagram(&cheat_pkt.encode_datagram().unwrap()).unwrap().expect("expected CanaryFraudReport");
    assert!(matches!(fraud_reply.body, ImpulseBody::MicroTrain(MicroTrainWireMessage::CanaryFraudReport(_))));
    let rep = auditor_node.compute_coordinator.escrow_vault.worker_reputation(&cheat_k.verifying_key().to_bytes());
    assert_eq!((rep.pouw_score_bps, rep.slashed_count), (0, 1));
}

#[test]
fn ten_slices_micro_train_pause_resume_and_canary_slashing() {
    let mut rng = ChaCha8Rng::seed_from_u64(202611);
    let (client_k, honest_k, cheat_k, auditor_k) = (
        SigningKey::generate(&mut rng), SigningKey::generate(&mut rng),
        SigningKey::generate(&mut rng), SigningKey::generate(&mut rng),
    );
    let (ledger, _) = UtxoDagLedger::from_genesis(vec![
        TxOutput::direct(honest_k.verifying_key().to_bytes(), NanoSkn::from_skn(200).unwrap()),
        TxOutput::direct(auditor_k.verifying_key().to_bytes(), NanoSkn::from_skn(200).unwrap()),
    ]).unwrap();
    let mut honest = AgentNode::from_signing_key("Honest", 10, honest_k.clone(), HardwareTier::StandardNode, ledger.clone(), SnowballParams::default()).unwrap();
    let mut auditor = AgentNode::from_signing_key("Auditor", 11, auditor_k, HardwareTier::StandardNode, ledger, SnowballParams::default()).unwrap();

    let (task_ten, cfg) = ([0x10; 32], MicroTrainConfig { dim_in: 4, dim_hidden: 6, dim_out: 2, inner_steps: 6, ..Default::default() });
    let mut vault = ComputeEscrowVault::new();
    vault.lock_task_simple(task_ten, client_k.verifying_key().to_bytes(), honest.pubkey, NanoSkn(100_000), NanoSkn(20_000), 100).unwrap();
    vault.register_chunk_schedule(task_ten, 10).unwrap();

    for idx in 0..10u16 {
        let (m, raw) = build_shard_manifest_and_bytes((task_ten, client_k.verifying_key().to_bytes(), idx), &cfg);
        let claim = stream_shard_to_worker(&mut honest, &client_k, m, &raw);
        assert!(claim.final_loss_micro < claim.initial_loss_micro);
        let receipt = vault.release_chunk_milestone(task_ten, idx, &honest.pubkey).unwrap();
        if idx == 9 { assert!(receipt.is_final_chunk && receipt.cumulative_released == NanoSkn(100_000)); }
    }
    assert_eq!(honest.opportunistic_worker.completed_shards, 10);

    let pause_batch = generate_canary_batch(&[0x77; 32], 1, 0, &cfg);
    honest.opportunistic_worker.update_telemetry(&SystemTelemetrySnapshot { active_game_detected: true, observed_at_sec: 10, ..Default::default() });
    let ShardExecutionStep::Preempted(ckpt) = honest.opportunistic_worker.run_or_pause_shard(&honest_k, &cfg, &pause_batch).unwrap() else { panic!("preempt"); };
    honest.opportunistic_worker.restore_checkpoint(ckpt).unwrap();
    honest.opportunistic_worker.update_telemetry(&SystemTelemetrySnapshot { observed_at_sec: 15, ..Default::default() });
    honest.opportunistic_worker.update_telemetry(&SystemTelemetrySnapshot { observed_at_sec: 50, ..Default::default() });
    assert!(matches!(honest.opportunistic_worker.run_or_pause_shard(&honest_k, &cfg, &pause_batch).unwrap(), ShardExecutionStep::Completed { .. }));

    assert_canary_catches_dishonest_worker(&mut auditor, &client_k, &cheat_k, &cfg);
}

fn build_bonded_genesis(
    humans: (&HumanWallet, &HumanWallet),
    pks: ([u8; 32], [u8; 32]),
) -> (UtxoDagLedger, GenesisHalvingVault, (UtxoId, TxOutput)) {
    let (client_bond_amt, worker_reserve) = (NanoSkn::from_skn(500).unwrap(), NanoSkn::from_skn(100).unwrap());
    let treasury_rem = NanoSkn(GENESIS_CIRCULATING_NANO_SKN).checked_sub(client_bond_amt.checked_add(worker_reserve).unwrap()).unwrap();
    let genesis = GenesisConfig {
        network_name: DEFAULT_MAINNET_NETWORK_NAME.to_string(),
        total_supply: NanoSkn(GENESIS_TOTAL_NANO_SKN), vault_allocation: NanoSkn(GENESIS_VAULT_NANO_SKN),
        circulating_allocation: NanoSkn(GENESIS_CIRCULATING_NANO_SKN),
        initial_utxos: vec![
            GenesisUtxoEntry { label: "client-bond".into(), owner_pubkey: pks.0, amount: client_bond_amt, policy: CustodyPolicy::AgentBond { human_owner: humans.0.human_pubkey(), max_voucher_allowance: NanoSkn::from_skn(200).unwrap() } },
            GenesisUtxoEntry { label: "worker-bond".into(), owner_pubkey: pks.1, amount: worker_reserve, policy: CustodyPolicy::AgentBond { human_owner: humans.1.human_pubkey(), max_voucher_allowance: NanoSkn::from_skn(20).unwrap() } },
            GenesisUtxoEntry::direct("bootstrap-treasury", [0x99; 32], treasury_rem),
        ],
        annual_halving_epochs: 365,
    };
    validate_genesis_invariants(&genesis).unwrap();
    let (ledger, halving_vault, _) = genesis.build_ledger_and_vault().unwrap();
    let bond = ledger.utxos_for_owner(&pks.0).into_iter().next().unwrap();
    (ledger, halving_vault, bond)
}

#[test]
fn unilateral_voucher_at_slice_six_and_yield_sweep_conserve_exact_21t_skn() {
    let (h_client, h_worker) = (HumanWallet::from_recovery_phrase("client-owner-cold-seed-006"), HumanWallet::from_recovery_phrase("gamer-worker-cold-seed-006"));
    let (client_agent_k, _) = h_client.authorize_agent(0, NanoSkn::from_skn(200).unwrap(), 1_000);
    let (worker_agent_k, _) = h_worker.authorize_agent(0, NanoSkn::from_skn(20).unwrap(), 1_000);
    let (client_pk, worker_pk) = (client_agent_k.verifying_key().to_bytes(), worker_agent_k.verifying_key().to_bytes());
    let worker_reserve = NanoSkn::from_skn(100).unwrap();
    let (mut ledger, halving_vault, (client_bond_id, client_bond_out)) = build_bonded_genesis((&h_client, &h_worker), (client_pk, worker_pk));

    let mut engine = OpportunisticWorkerEngine::default().with_sweep_config(YieldSweepConfig::new(h_worker.human_pubkey(), worker_reserve));
    let (mut escrow, task_id, cfg) = (ComputeEscrowVault::new(), [0x66; 32], MicroTrainConfig { dim_in: 4, dim_hidden: 6, dim_out: 2, inner_steps: 4, ..Default::default() });
    escrow.lock_task_simple(task_id, client_pk, worker_pk, NanoSkn::from_skn(100).unwrap(), NanoSkn::from_skn(20).unwrap(), 100).unwrap();
    let sched = escrow.register_chunk_schedule(task_id, 10).unwrap();

    for idx in 0..6u16 {
        let batch = generate_canary_batch(&[idx as u8; 32], 1, idx, &cfg);
        assert!(matches!(engine.run_or_pause_shard(&worker_agent_k, &cfg, &batch).unwrap(), ShardExecutionStep::Completed { .. }));
        if matches!(idx, 1 | 2 | 4) { continue; }
        let v = SynapticVoucher::new_signed(&client_agent_k, client_bond_id, worker_pk, u64::from(idx) + 1, sched.cumulative_target_for_chunk(idx).unwrap());
        engine.receive_chunk_voucher(ChunkVoucherDelivery { task_id, chunk_index: idx, total_chunks: 10, voucher: v }, NanoSkn::from_skn(200).unwrap()).unwrap();
    }

    let tracker = engine.tracker_for_bond(&client_bond_id).unwrap();
    let uni_escrow = escrow.settle_unilateral_with_voucher(task_id, tracker.highest_voucher.as_ref().unwrap(), NanoSkn::from_skn(200).unwrap()).unwrap();
    assert_eq!((uni_escrow.total_worker_earned, uni_escrow.client_unspent_refund), (NanoSkn::from_skn(60).unwrap(), NanoSkn::from_skn(40).unwrap()));

    let (settle_tx, v_proof) = tracker.build_unilateral_settlement_tx(&worker_agent_k, &client_bond_out, NanoSkn::ONE_NANO, vec![client_bond_id.tx_hash], 1).unwrap();
    let (stx_hash, _) = ledger.admit_unilateral_voucher_settlement(settle_tx, &v_proof).unwrap();
    ledger.finalize_tx(&stx_hash).unwrap();
    let merge_tx = engine.consolidate_direct_to_bond_tx(&worker_agent_k, &ledger).unwrap().expect("consolidate to bond");
    let (mtx_hash, _) = ledger.admit_pending_tx(merge_tx).unwrap();
    ledger.finalize_tx(&mtx_hash).unwrap();
    let sweep_tx = engine.try_build_yield_sweep(&worker_agent_k, &ledger).unwrap().expect("60 SKN >= 50 SKN auto-sweep threshold");
    assert_eq!(sweep_tx.fee, NanoSkn::ZERO);
    let (sw_hash, _) = ledger.admit_pending_tx(sweep_tx).unwrap();
    ledger.finalize_tx(&sw_hash).unwrap();
    assert_eq!((ledger.balance_of(&h_worker.human_pubkey()), ledger.balance_of(&worker_pk)), (NanoSkn::from_skn(60).unwrap(), worker_reserve));
    assert_eq!(halving_vault.remaining_reserve.checked_add(ledger.total_tracked_supply().unwrap()).unwrap(), NanoSkn(GENESIS_TOTAL_NANO_SKN));
}
