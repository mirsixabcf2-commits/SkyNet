//! `classic_sim`: Multi-Agent multicellular Candle evolution, 25% lossy UDP
//! `SynapticVoucher` micro-payments, `Utreexo` witness verification, and `Snowball` finality.

use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::Duration;

use ed25519_dalek::SigningKey;
use skynet_agent::{AgentNode, SnowballParams};
use skynet_cell::HardwareTier;
use skynet_ledger::{
    GenesisHalvingVault, HumanWallet, NanoSkn, SknTransaction, SynapticVoucher, TxOutput,
    UtxoDagLedger, UtxoId,
};
use skynet_wire::{ImpulseBody, ImpulsePacket};
use tokio::net::UdpSocket;

use crate::sim_metrics::{collect_growth_rows, ClassicSimReport, SimError};

const SIMULATED_UDP_PACKET_LOSS: f64 = 0.25;
const EVOLUTION_TICKS: usize = 16;
const VOUCHER_STREAM_COUNT: u64 = 40;
const MAX_SNOWBALL_ROUNDS: u32 = 50;

struct ClassicCluster {
    human_alice: HumanWallet, human_bob: HumanWallet, max_allowance: NanoSkn,
    keys: Vec<SigningKey>, agents: Vec<AgentNode>, sockets: Vec<UdpSocket>, addrs: Vec<SocketAddr>,
    base_ledger: UtxoDagLedger, genesis_hash: [u8; 32], seed_circulating: NanoSkn, halving_vault: GenesisHalvingVault,
}

pub async fn run_classic_simulation() -> Result<ClassicSimReport, SimError> {
    let mut c = bootstrap_classic_cluster().await?;
    let (halving_y0, halving_y1) = (c.halving_vault.annual_quota_for_year(0), c.halving_vault.annual_quota_for_year(1));
    let _ = c.halving_vault.release_next_epoch()?;
    let total_accounted_nano = c.halving_vault.total_accounted_supply()?.0;

    run_cellular_evolution(&mut c.agents)?;
    let agent_rows = collect_growth_rows(&c.agents);

    stream_lossy_vouchers(&mut c).await?;
    let (contested_utxo, witnessed_wire_size) = seed_double_spend_conflict(&mut c)?;
    run_snowball_rounds(&mut c, &contested_utxo).await?;
    exchange_plasmids_and_mitosis(&mut c).await?;

    let unanimous_winner = verify_unanimous_finality(&c, &contested_utxo)?;
    Ok(ClassicSimReport {
        halving_y0, halving_y1, total_accounted_nano,
        alice_net_worth: c.base_ledger.human_total_net_worth(&c.human_alice.human_pubkey()),
        utreexo_ram_bytes: c.base_ledger.build_utreexo_forest().roots.ram_footprint_bytes(),
        agent_rows, witnessed_wire_size,
        voucher_earned_nano: c.agents[1].total_voucher_nano_earned.0,
        udp_sent: c.agents.iter().map(|a| a.udp_sent).sum(),
        udp_dropped: c.agents.iter().map(|a| a.udp_dropped_simulated).sum(),
        unanimous_winner, supply_conserved: true,
    })
}

const CLASSIC_SPECS: [(&str, HardwareTier, u128, bool, u32); 6] = [
    ("Titan-GPU-01", HardwareTier::ColossalOrganism, 500_000, true, 0),
    ("Titan-GPU-02", HardwareTier::ColossalOrganism, 500_000, false, 0),
    ("Laptop-Node-03", HardwareTier::StandardNode, 150_000, true, 1),
    ("Laptop-Node-04", HardwareTier::StandardNode, 150_000, false, 1),
    ("IoT-Edge-05", HardwareTier::TinyPeripheral, 25_000, true, 2),
    ("Periph-Sensor-06", HardwareTier::TinyPeripheral, 25_000, false, 2),
];

async fn bootstrap_classic_cluster() -> Result<ClassicCluster, SimError> {
    let human_alice = HumanWallet::from_recovery_phrase("alice-human-cold-seed-skynet-2026");
    let human_bob = HumanWallet::from_recovery_phrase("bob-human-cold-seed-skynet-2026");
    let max_allowance = NanoSkn::from_skn(10)?;
    let (mut keys, mut certs, mut genesis_outputs, mut seed_circulating) = (Vec::new(), Vec::new(), Vec::new(), NanoSkn::ZERO);
    for (_, _, skn_whole, is_alice, sub_idx) in &CLASSIC_SPECS {
        let owner = if *is_alice { &human_alice } else { &human_bob };
        let (hotkey, cert) = owner.authorize_agent(*sub_idx, max_allowance, 10_000);
        let amt = NanoSkn::from_skn(*skn_whole)?.checked_add(NanoSkn::from_nano(7))?;
        seed_circulating = seed_circulating.checked_add(amt)?;
        genesis_outputs.push(TxOutput::agent_bond(owner.human_pubkey(), cert.agent_pubkey, amt, max_allowance));
        keys.push(hotkey); certs.push(cert);
    }
    let cold_each = NanoSkn::from_skn(100_000)?;
    for owner in [&human_alice, &human_bob] { seed_circulating = seed_circulating.checked_add(cold_each)?; genesis_outputs.push(TxOutput::direct(owner.human_pubkey(), cold_each)); }
    let halving_vault = GenesisHalvingVault::new(seed_circulating, 365)?;
    let (base_ledger, genesis_hash) = UtxoDagLedger::from_genesis(genesis_outputs)?;
    let sb = SnowballParams { sample_k: 5, quorum_alpha: 2, finality_beta: 5 };
    let (mut agents, mut sockets, mut addrs) = (Vec::new(), Vec::new(), Vec::new());
    for (i, (name, tier, _, _, _)) in CLASSIC_SPECS.iter().enumerate() {
        let sock = UdpSocket::bind("127.0.0.1:0").await?;
        addrs.push(sock.local_addr()?); sockets.push(sock);
        let mut ag = AgentNode::from_signing_key(*name, 1000 + i as u64, keys[i].clone(), *tier, base_ledger.clone(), sb)?;
        ag.ownership_cert = Some(certs[i].clone());
        agents.push(ag);
    }
    Ok(ClassicCluster { human_alice, human_bob, max_allowance, keys, agents, sockets, addrs, base_ledger, genesis_hash, seed_circulating, halving_vault })
}

fn run_cellular_evolution(agents: &mut [AgentNode]) -> Result<(), SimError> {
    let stimulus = [0.65, -0.30, 0.80, 0.15, -0.45, 0.70, 0.25, -0.60];
    let target = [0.75, -0.65, 0.80, -0.55];
    for _ in 0..EVOLUTION_TICKS {
        for ag in agents.iter_mut() { let _ = ag.run_biological_tick(&stimulus, &target)?; }
    }
    Ok(())
}

async fn stream_lossy_vouchers(c: &mut ClassicCluster) -> Result<(), SimError> {
    let stimulus = [0.65, -0.30, 0.80, 0.15, -0.45, 0.70, 0.25, -0.60];
    let (bond, payee) = (UtxoId { tx_hash: c.genesis_hash, vout: 4 }, c.agents[1].pubkey);
    for seq in 1..=VOUCHER_STREAM_COUNT {
        let voucher = SynapticVoucher::new_signed(&c.keys[4], bond, payee, seq, NanoSkn::from_nano(u128::from(seq) * 50));
        let pkt = ImpulsePacket::new_signed(&c.keys[4], 0, 0, 3, 0, seq as u32, ImpulseBody::SynapticMicroPay { voucher, stimulus })?;
        let loss = if seq == VOUCHER_STREAM_COUNT { 0.0 } else { SIMULATED_UDP_PACKET_LOSS };
        let _ = c.agents[4].send_lossy_udp(&c.sockets[4], c.addrs[1], &pkt, loss).await?;
    }
    tokio::time::sleep(Duration::from_millis(8)).await;
    drain_socket_into_agent(&c.sockets[1], &mut c.agents[1])
}

fn seed_double_spend_conflict(c: &mut ClassicCluster) -> Result<(UtxoId, usize), SimError> {
    let (contested, send_amt, fee) = (UtxoId { tx_hash: c.genesis_hash, vout: 0 }, NanoSkn::from_nano(1_234_567_891), NanoSkn::ONE_NANO);
    let rem_bond = c.base_ledger.balance_of(&c.agents[0].pubkey).checked_sub(send_amt)?.checked_sub(fee)?;
    let bond_change = TxOutput::agent_bond(c.human_alice.human_pubkey(), c.agents[0].pubkey, rem_bond, c.max_allowance.checked_sub(send_amt)?);
    let tx_a = SknTransaction::new_signed(&c.keys[0], vec![contested], vec![TxOutput::direct(c.human_bob.human_pubkey(), send_amt), bond_change.clone()], fee, vec![c.genesis_hash], 1);
    let tx_b = SknTransaction::new_signed(&c.keys[0], vec![contested], vec![TxOutput::direct(c.agents[5].pubkey, send_amt), bond_change], fee, vec![c.genesis_hash], 2);

    let spent_out = c.base_ledger.get_utxo(&contested).ok_or(SimError::Invariant("missing contested UTXO"))?.clone();
    let proof = c.base_ledger.build_utreexo_forest().prove(&contested).ok_or(SimError::Invariant("missing Utreexo proof"))?;
    let raw_w = ImpulsePacket::new_signed(&c.keys[0], 0, 0, 3, 0, 1, ImpulseBody::WitnessedTx { tx: tx_a.clone(), spent_outputs: vec![spent_out], proofs: vec![proof] })?.encode_datagram()?;
    for (idx, ag) in c.agents.iter_mut().enumerate() {
        if idx < 4 {
            let _ = ag.handle_incoming_datagram(&raw_w)?;
            ag.ledger.admit_pending_tx(tx_b.clone())?;
        } else {
            ag.ledger.admit_pending_tx(tx_b.clone())?;
            ag.snowball.register_tx(&tx_b);
            ag.ledger.admit_pending_tx(tx_a.clone())?;
        }
    }
    Ok((contested, raw_w.len()))
}

async fn run_snowball_rounds(c: &mut ClassicCluster, contested: &UtxoId) -> Result<(), SimError> {
    for round in 0..MAX_SNOWBALL_ROUNDS {
        emit_snowball_queries(c, round).await?;
        tokio::time::sleep(Duration::from_millis(6)).await;
        reply_snowball_queries(c).await?;
        tokio::time::sleep(Duration::from_millis(6)).await;
        for i in 0..c.agents.len() {
            drain_socket_into_agent(&c.sockets[i], &mut c.agents[i])?;
            let _ = c.agents[i].finalize_snowball_pulse()?;
        }
        if c.agents.iter().all(|a| a.snowball.decided_for(contested).is_some()) { break; }
    }
    Ok(())
}

async fn emit_snowball_queries(c: &mut ClassicCluster, round: u32) -> Result<(), SimError> {
    let all_pks: Vec<[u8; 32]> = c.agents.iter().map(|a| a.pubkey).collect();
    for i in 0..c.agents.len() {
        let contested = c.agents[i].snowball.contested_utxos();
        if contested.is_empty() { continue; }
        let peers: Vec<SocketAddr> = c.addrs.iter().enumerate().filter(|(idx, _)| *idx != i).map(|(_, a)| *a).collect();
        let targets = c.agents[i].sample_peers(&peers);
        let qid = (i as u64) * 1000 + u64::from(round);
        let queried: HashSet<[u8; 32]> = targets.iter().filter_map(|t| c.addrs.iter().position(|a| a == t).map(|p| all_pks[p])).collect();
        c.agents[i].initiate_snowball_query(qid, queried, contested.clone());
        let pkt = ImpulsePacket::new_signed(&c.agents[i].signing_key, c.agents[i].cluster_prefix, c.agents[i].prefix_len, 3, c.agents[i].epoch, round, ImpulseBody::SnowballPull { query_id: qid, contested_utxos: contested })?;
        for addr in targets {
            let _ = c.agents[i].send_lossy_udp(&c.sockets[i], addr, &pkt, SIMULATED_UDP_PACKET_LOSS).await?;
        }
    }
    Ok(())
}

async fn reply_snowball_queries(c: &mut ClassicCluster) -> Result<(), SimError> {
    let mut buf = [0u8; 2048];
    for i in 0..c.agents.len() {
        while let Ok((len, src)) = c.sockets[i].try_recv_from(&mut buf) {
            if let Some(reply) = c.agents[i].handle_incoming_datagram(&buf[..len])? {
                let _ = c.agents[i].send_lossy_udp(&c.sockets[i], src, &reply, SIMULATED_UDP_PACKET_LOSS).await?;
            }
        }
    }
    Ok(())
}

async fn exchange_plasmids_and_mitosis(c: &mut ClassicCluster) -> Result<(), SimError> {
    for pkt in &c.agents[0].encode_elite_plasmid_droplets(18)? {
        let _ = c.agents[0].send_lossy_udp(&c.sockets[0], c.addrs[4], pkt, SIMULATED_UDP_PACKET_LOSS).await?;
    }
    tokio::time::sleep(Duration::from_millis(10)).await;
    drain_socket_into_agent(&c.sockets[4], &mut c.agents[4])?;
    for ag in &mut c.agents {
        ag.split_cluster_prefix();
        ag.verify_internal_skn_conservation()?;
    }
    Ok(())
}

fn drain_socket_into_agent(sock: &UdpSocket, ag: &mut AgentNode) -> Result<(), SimError> {
    let mut buf = [0u8; 2048];
    while let Ok((len, _)) = sock.try_recv_from(&mut buf) { let _ = ag.handle_incoming_datagram(&buf[..len])?; }
    Ok(())
}

fn verify_unanimous_finality(c: &ClassicCluster, contested: &UtxoId) -> Result<[u8; 32], SimError> {
    let winner = c.agents[0].snowball.decided_for(contested).ok_or(SimError::Invariant("Agent 0 failed Snowball finality"))?;
    for ag in &c.agents {
        if ag.snowball.decided_for(contested) != Some(winner) || ag.ledger.total_tracked_supply()? != c.seed_circulating {
            return Err(SimError::Invariant("Snowball finality or SKN conservation violated"));
        }
    }
    Ok(winner)
}
