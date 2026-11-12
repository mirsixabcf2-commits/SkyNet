//! `opportunistic_sim`: Chaos simulation verifying 40% UDP packet loss `FountainDroplet`
//! shard recovery, `< 100ms` gamer preemption + `MicroTrainCheckpoint` resume, 30%
//! Byzantine slashing + `TrimmedMeanAggregator`, and exact `21T SKN` `Yield Sweep`.

use std::time::{Duration, Instant};

use ed25519_dalek::SigningKey;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use skynet_agent::{AgentNode, OpportunisticWorkerEngine, OpportunisticWorkerError, ShardExecutionStep, SnowballParams, SystemTelemetrySnapshot};
use skynet_cell::{compute_canonical_digest, decode_dense_f16, encode_dense_f16, execute_micro_train_slices, generate_canary_batch, HardwareTier, MicroTrainBatch, MicroTrainConfig, MicroTrainOutcome, PreemptGuard, TrimmedMeanAggregator, WeightedDeltaSubmission};
use skynet_dna::{apply_plasmid_patch, CellDna, CellRole};
use skynet_ledger::compute_escrow::ComputeEscrowVault;
use skynet_ledger::{validate_genesis_invariants, CustodyPolicy, GenesisConfig, GenesisHalvingVault, GenesisUtxoEntry, HumanWallet, NanoSkn, SynapticVoucher, UtxoDagLedger, UtxoId, YieldSweepConfig, DEFAULT_MAINNET_NETWORK_NAME, GENESIS_CIRCULATING_NANO_SKN, GENESIS_TOTAL_NANO_SKN, GENESIS_VAULT_NANO_SKN};
use skynet_wire::{encode_micro_train_shard, ChunkVoucherDelivery, ImpulseBody, ImpulsePacket, MicroBatchManifest, MicroTrainWireMessage, WeightDeltaClaim};
use tokio::net::UdpSocket;

use crate::sim_metrics::{OpportunisticSimReport, SimError};

const CHAOS_F40_LOSS: f64 = 0.40;
const TOTAL_WORKERS: usize = 10;
const BYZANTINE_WORKERS: usize = 3;

pub async fn run_opportunistic_simulation() -> Result<OpportunisticSimReport, SimError> {
    let mut rng = ChaCha8Rng::seed_from_u64(2_026_006);
    let (client_k, worker_k, auditor_k) = (SigningKey::generate(&mut rng), SigningKey::generate(&mut rng), SigningKey::generate(&mut rng));
    let (mut ledger, halving_vault, client_bond_id) = build_21t_genesis(&client_k, &worker_k)?;
    let (droplets_sent, droplets_dropped, max_wire_bytes) = simulate_40pct_loss_fountain_udp(&client_k, &worker_k, &ledger).await?;
    let (preempt_latency_ms, resumed_steps) = simulate_gamer_preemption_and_resume(&worker_k)?;
    let (byzantine_slashed, initial_loss, converged_loss) = simulate_30pct_byzantine_canary_and_agg(&mut rng, &client_k, &auditor_k, &ledger)?;
    let (swept_to_cold_skn, worker_reserve_skn) = simulate_streaming_and_yield_sweep(&mut ledger, client_bond_id)?;
    let total_accounted_nano = halving_vault.remaining_reserve.checked_add(ledger.total_tracked_supply()?)?.0;

    Ok(OpportunisticSimReport {
        droplets_sent, droplets_dropped, max_wire_bytes, preempt_latency_ms, resumed_steps,
        total_workers: TOTAL_WORKERS, byzantine_slashed, initial_loss, converged_loss,
        swept_to_cold_skn, worker_reserve_skn, total_accounted_nano,
    })
}

fn build_21t_genesis(client_fb: &SigningKey, worker_fb: &SigningKey) -> Result<(UtxoDagLedger, GenesisHalvingVault, UtxoId), SimError> {
    let (h_client, h_worker) = (HumanWallet::from_recovery_phrase("opp-sim-client-cold-2026"), HumanWallet::from_recovery_phrase("opp-sim-gamer-cold-2026"));
    let (c_key, _) = h_client.authorize_agent(0, NanoSkn::from_skn(200)?, 1_000);
    let (w_key, _) = h_worker.authorize_agent(0, NanoSkn::from_skn(20)?, 1_000);
    let (c_amt, w_amt) = (NanoSkn::from_skn(500)?, NanoSkn::from_skn(100)?);
    let rem = NanoSkn(GENESIS_CIRCULATING_NANO_SKN).checked_sub(c_amt.checked_add(w_amt)?)?.checked_sub(NanoSkn::from_skn(20)?)?;
    let genesis = GenesisConfig {
        network_name: DEFAULT_MAINNET_NETWORK_NAME.to_string(),
        total_supply: NanoSkn(GENESIS_TOTAL_NANO_SKN), vault_allocation: NanoSkn(GENESIS_VAULT_NANO_SKN), circulating_allocation: NanoSkn(GENESIS_CIRCULATING_NANO_SKN),
        initial_utxos: vec![
            GenesisUtxoEntry { label: "client-bond".into(), owner_pubkey: c_key.verifying_key().to_bytes(), amount: c_amt, policy: CustodyPolicy::AgentBond { human_owner: h_client.human_pubkey(), max_voucher_allowance: NanoSkn::from_skn(200)? } },
            GenesisUtxoEntry { label: "worker-bond".into(), owner_pubkey: w_key.verifying_key().to_bytes(), amount: w_amt, policy: CustodyPolicy::AgentBond { human_owner: h_worker.human_pubkey(), max_voucher_allowance: NanoSkn::from_skn(20)? } },
            GenesisUtxoEntry::direct("sim-client", client_fb.verifying_key().to_bytes(), NanoSkn::from_skn(10)?),
            GenesisUtxoEntry::direct("sim-worker", worker_fb.verifying_key().to_bytes(), NanoSkn::from_skn(10)?),
            GenesisUtxoEntry::direct("treasury", [0x88; 32], rem),
        ],
        annual_halving_epochs: 365,
    };
    validate_genesis_invariants(&genesis)?;
    let (ledger, vault, _) = genesis.build_ledger_and_vault()?;
    let bond_id = ledger.utxos_for_owner(&c_key.verifying_key().to_bytes()).first().ok_or(SimError::Invariant("missing client bond"))?.0;
    Ok((ledger, vault, bond_id))
}

async fn simulate_40pct_loss_fountain_udp(client_k: &SigningKey, worker_k: &SigningKey, ledger: &UtxoDagLedger) -> Result<(u32, u32, usize), SimError> {
    let (sock_c, sock_w) = (UdpSocket::bind("127.0.0.1:0").await?, UdpSocket::bind("127.0.0.1:0").await?);
    let w_addr = sock_w.local_addr()?;
    let mut worker = AgentNode::from_signing_key("Gamer-GPU", 77, worker_k.clone(), HardwareTier::StandardNode, ledger.clone(), SnowballParams::default())?;
    let cfg = MicroTrainConfig { dim_in: 8, dim_hidden: 12, dim_out: 4, inner_steps: 6, ..Default::default() };
    let mut batch = generate_canary_batch(&[0x42; 32], 1, 0, &cfg);
    batch.samples_x = batch.samples_x.repeat(16);
    batch.targets_y = batch.targets_y.repeat(16);
    let raw = encode_batch_bincode_v1(&batch);
    let shard_hash = encode_micro_train_shard(&raw)?.next_droplet().object_hash;
    let m = MicroBatchManifest {
        task_id: batch.task_id, client_pubkey: client_k.verifying_key().to_bytes(), opcode: 0x1301, shard_id: 0, total_shards: 1,
        dim_in: 8, dim_hidden: 12, dim_out: 4, inner_steps: 6, learning_rate_micro: cfg.learning_rate_micro,
        shard_object_hash: shard_hash, shard_bytes_len: raw.len() as u32, bounty_per_chunk_nano: 25_000, min_worker_bond_nano: 1_000,
    };
    let m_wire = ImpulsePacket::new_signed(client_k, 0, 0, 4, 1, 1, ImpulseBody::MicroTrain(MicroTrainWireMessage::MicroBatchManifest(m)))?.encode_datagram()?;
    let _ = worker.handle_incoming_datagram(&m_wire)?;
    stream_droplets_over_udp(&sock_c, &sock_w, w_addr, (&mut worker, client_k, &raw, m_wire.len())).await
}

async fn stream_droplets_over_udp(sock_c: &UdpSocket, sock_w: &UdpSocket, w_addr: std::net::SocketAddr, ctx: (&mut AgentNode, &SigningKey, &[u8], usize)) -> Result<(u32, u32, usize), SimError> {
    let (worker, client_k, raw, mut max_wire) = ctx;
    let (mut enc, mut rng) = (encode_micro_train_shard(raw)?, ChaCha8Rng::seed_from_u64(404040));
    let (mut sent, mut dropped, mut buf) = (0u32, 0u32, [0u8; 2048]);
    for seq in 0..160u32 {
        let wire = ImpulsePacket::new_signed(client_k, 0, 0, 4, 1, seq + 2, ImpulseBody::FountainDroplet(enc.next_droplet()))?.encode_datagram()?;
        max_wire = max_wire.max(wire.len());
        sent += 1;
        if rng.gen_bool(CHAOS_F40_LOSS) { dropped += 1; continue; }
        sock_c.send_to(&wire, w_addr).await?;
        tokio::time::sleep(Duration::from_micros(250)).await;
        if drain_converged_claim(sock_w, worker, &mut buf, &mut max_wire)? {
            return Ok((sent, dropped, max_wire));
        }
    }
    Err(SimError::Invariant("Fountain shard failed to recover under 40% UDP loss"))
}

fn drain_converged_claim(sock_w: &UdpSocket, worker: &mut AgentNode, buf: &mut [u8; 2048], max_wire: &mut usize) -> Result<bool, SimError> {
    while let Ok((n, _)) = sock_w.try_recv_from(buf) {
        let Some(reply) = worker.handle_incoming_datagram(&buf[..n])? else { continue };
        *max_wire = (*max_wire).max(reply.encode_datagram()?.len());
        let ImpulseBody::MicroTrain(MicroTrainWireMessage::WeightDeltaClaim(c)) = reply.body else { continue };
        if c.final_loss_micro < c.initial_loss_micro { return Ok(true); }
    }
    Ok(false)
}

fn simulate_gamer_preemption_and_resume(worker_k: &SigningKey) -> Result<(f64, u16), SimError> {
    let cfg = MicroTrainConfig { dim_in: 4, dim_hidden: 8, dim_out: 2, inner_steps: 8, ..Default::default() };
    let (batch, mut eng, t0) = (generate_canary_batch(&[0x55; 32], 2, 1, &cfg), OpportunisticWorkerEngine::default(), Instant::now());
    let _ = eng.update_telemetry(&SystemTelemetrySnapshot { active_game_detected: true, observed_at_sec: 10, ..Default::default() });
    let ShardExecutionStep::Preempted(ckpt) = eng.run_or_pause_shard(worker_k, &cfg, &batch)? else {
        return Err(SimError::Invariant("expected immediate gamer preemption"));
    };
    let preempt_ms = t0.elapsed().as_secs_f64() * 1_000.0;
    ckpt.verify_integrity().map_err(OpportunisticWorkerError::Checkpoint)?;
    eng.restore_checkpoint(ckpt)?;
    let _ = eng.update_telemetry(&SystemTelemetrySnapshot { observed_at_sec: 15, ..Default::default() });
    let _ = eng.update_telemetry(&SystemTelemetrySnapshot { observed_at_sec: 50, ..Default::default() });
    let ShardExecutionStep::Completed { result, .. } = eng.run_or_pause_shard(worker_k, &cfg, &batch)? else {
        return Err(SimError::Invariant("expected checkpoint resume completion"));
    };
    Ok((preempt_ms, result.completed_steps))
}

fn simulate_30pct_byzantine_canary_and_agg(rng: &mut ChaCha8Rng, client_k: &SigningKey, auditor_k: &SigningKey, ledger: &UtxoDagLedger) -> Result<(usize, f32, f32), SimError> {
    let dna = CellDna::genesis_stem(303).differentiate(CellRole::Neural);
    let cfg = MicroTrainConfig { dim_in: 8, dim_hidden: 16, dim_out: 4, inner_steps: 6, ..Default::default() };
    let batch = generate_canary_batch(&[0x9A; 32], 3, 0, &cfg);
    let MicroTrainOutcome::Completed(ref_res) = execute_micro_train_slices(&cfg, &batch, None, &PreemptGuard::new())? else {
        return Err(SimError::Invariant("canary reference did not complete"));
    };
    let mut auditor = AgentNode::from_signing_key("Auditor", 88, auditor_k.clone(), HardwareTier::StandardNode, ledger.clone(), SnowballParams::default())?;
    auditor.opportunistic_worker.register_canary_reference(ref_res.clone());
    let (mut subs, mut slashed) = (Vec::with_capacity(TOTAL_WORKERS), 0usize);
    for idx in 0..TOTAL_WORKERS {
        let w_key = SigningKey::generate(rng);
        if idx < TOTAL_WORKERS - BYZANTINE_WORKERS {
            subs.push(WeightedDeltaSubmission { worker_id: w_key.verifying_key().to_bytes(), delta_f16: ref_res.delta_f16.clone(), pouw_score_bps: 8_000 });
            continue;
        }
        slashed += slash_byzantine_worker(&mut auditor, client_k, &w_key, &ref_res)?;
        subs.push(WeightedDeltaSubmission { worker_id: w_key.verifying_key().to_bytes(), delta_f16: encode_dense_f16(&vec![250.0; ref_res.delta_f16.len()]), pouw_score_bps: 2_500 });
    }
    let (agg, patch) = TrimmedMeanAggregator::default().aggregate_to_plasmid_patch(&subs, dna.dna_id, 1_800)?;
    let _ = apply_plasmid_patch(&dna, &patch)?;
    let updated: Vec<f32> = decode_dense_f16(&batch.base_weights_f16).iter().zip(&agg.delta_f32).map(|(b, d)| b + d).collect();
    let eval_b = MicroTrainBatch { base_weights_f16: encode_dense_f16(&updated), ..batch };
    let MicroTrainOutcome::Completed(ev) = execute_micro_train_slices(&MicroTrainConfig { inner_steps: 1, ..cfg }, &eval_b, None, &PreemptGuard::new())? else {
        return Err(SimError::Invariant("eval step failed"));
    };
    Ok((slashed, ref_res.initial_loss, ev.initial_loss))
}

fn slash_byzantine_worker(auditor: &mut AgentNode, client_k: &SigningKey, cheat_k: &SigningKey, ref_res: &skynet_cell::MicroTrainResult) -> Result<usize, SimError> {
    let mut task_id = ref_res.task_id;
    task_id[31] ^= cheat_k.verifying_key().to_bytes()[0];
    auditor.compute_coordinator.escrow_vault.lock_task_simple(task_id, client_k.verifying_key().to_bytes(), cheat_k.verifying_key().to_bytes(), NanoSkn(40_000), NanoSkn(20_000), 100)?;
    let mut bad_ref = ref_res.clone();
    bad_ref.task_id = task_id;
    auditor.opportunistic_worker.register_canary_reference(bad_ref);
    let bad_f16 = encode_dense_f16(&ref_res.decode_delta_f32().into_iter().map(|v| -v).collect::<Vec<_>>());
    let claim = WeightDeltaClaim {
        task_id, shard_id: ref_res.shard_id, worker_pubkey: [0; 32], completed_steps: ref_res.completed_steps,
        initial_loss_micro: 900_000, final_loss_micro: 450_000, canonical_digest: compute_canonical_digest(&decode_dense_f16(&bad_f16)),
        inline_delta_f16: bad_f16, delta_object_hash: None, worker_signature: Vec::new(),
    }.into_signed(cheat_k)?;
    let pkt = ImpulsePacket::new_signed(cheat_k, 0, 0, 4, 1, 1, ImpulseBody::MicroTrain(MicroTrainWireMessage::WeightDeltaClaim(claim)))?;
    let _ = auditor.handle_incoming_datagram(&pkt.encode_datagram()?)?;
    let rep = auditor.compute_coordinator.escrow_vault.worker_reputation(&cheat_k.verifying_key().to_bytes());
    Ok(usize::from(rep.pouw_score_bps == 0 && rep.slashed_count == 1))
}

fn simulate_streaming_and_yield_sweep(ledger: &mut UtxoDagLedger, client_bond_id: UtxoId) -> Result<(u64, u64), SimError> {
    let (h_client, h_worker) = (HumanWallet::from_recovery_phrase("opp-sim-client-cold-2026"), HumanWallet::from_recovery_phrase("opp-sim-gamer-cold-2026"));
    let (c_key, _) = h_client.authorize_agent(0, NanoSkn::from_skn(200)?, 1_000);
    let (w_key, _) = h_worker.authorize_agent(0, NanoSkn::from_skn(20)?, 1_000);
    let (c_pk, w_pk, reserve) = (c_key.verifying_key().to_bytes(), w_key.verifying_key().to_bytes(), NanoSkn::from_skn(100)?);
    let bond_out = ledger.get_utxo(&client_bond_id).ok_or(SimError::Invariant("missing bond out"))?.clone();
    let mut eng = OpportunisticWorkerEngine::default().with_sweep_config(YieldSweepConfig::new(h_worker.human_pubkey(), reserve));
    let (mut escrow, task_id) = (ComputeEscrowVault::new(), [0xC6; 32]);
    escrow.lock_task_simple(task_id, c_pk, w_pk, NanoSkn::from_skn(100)?, NanoSkn::from_skn(20)?, 100)?;
    let sched = escrow.register_chunk_schedule(task_id, 10)?;
    for idx in [0u16, 3, 5] {
        let v = SynapticVoucher::new_signed(&c_key, client_bond_id, w_pk, u64::from(idx) + 1, sched.cumulative_target_for_chunk(idx)?);
        let _ = eng.receive_chunk_voucher(ChunkVoucherDelivery { task_id, chunk_index: idx, total_chunks: 10, voucher: v }, NanoSkn::from_skn(200)?)?;
    }
    let tr = eng.tracker_for_bond(&client_bond_id).ok_or(SimError::Invariant("missing tracker"))?;
    let hv = tr.highest_voucher.as_ref().ok_or(SimError::Invariant("missing highest voucher"))?;
    let _ = escrow.settle_unilateral_with_voucher(task_id, hv, NanoSkn::from_skn(200)?)?;
    let (stx, proof) = tr.build_unilateral_settlement_tx(&w_key, &bond_out, NanoSkn::ONE_NANO, vec![client_bond_id.tx_hash], 1)?;
    let (sh, _) = ledger.admit_unilateral_voucher_settlement(stx, &proof)?;
    ledger.finalize_tx(&sh)?;
    let mtx = eng.consolidate_direct_to_bond_tx(&w_key, ledger)?.ok_or(SimError::Invariant("missing consolidate tx"))?;
    let (mh, _) = ledger.admit_pending_tx(mtx)?;
    ledger.finalize_tx(&mh)?;
    let sw = eng.try_build_yield_sweep(&w_key, ledger)?.ok_or(SimError::Invariant("missing yield sweep tx"))?;
    let (swh, _) = ledger.admit_pending_tx(sw)?;
    ledger.finalize_tx(&swh)?;
    Ok((ledger.balance_of(&h_worker.human_pubkey()).whole_skn() as u64, ledger.balance_of(&w_pk).whole_skn() as u64))
}

fn encode_batch_bincode_v1(b: &MicroTrainBatch) -> Vec<u8> {
    let mut out = Vec::with_capacity(64 + b.base_weights_f16.len() * 2 + (b.samples_x.len() + b.targets_y.len()) * 4);
    out.extend_from_slice(&b.task_id);
    out.extend_from_slice(&b.shard_id.to_le_bytes());
    out.extend_from_slice(&(b.base_weights_f16.len() as u64).to_le_bytes());
    for &w in &b.base_weights_f16 { out.extend_from_slice(&w.to_le_bytes()); }
    out.extend_from_slice(&(b.samples_x.len() as u64).to_le_bytes());
    for &x in &b.samples_x { out.extend_from_slice(&x.to_le_bytes()); }
    out.extend_from_slice(&(b.targets_y.len() as u64).to_le_bytes());
    for &y in &b.targets_y { out.extend_from_slice(&y.to_le_bytes()); }
    out
}
