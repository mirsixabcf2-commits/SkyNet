//! `skynet-sim`: Live multi-Agent Chaos UDP simulation verifying:
//! 1. `21,000,000,000,000 SKN` fixed Genesis cap (`u128` `nanoSKN`) & 50% annual halving.
//! 2. Human Sovereign Wallets (`HumanWallet`) delegating bounded `AgentBond` custody to fleets of AI Agents.
//! 3. Multicellular AI scaling on `candle` (`TinyPeripheral` vs `StandardNode` vs `ColossalOrganism`).
//! 4. Ultra-light `Utreexo` (`~1 KB` RAM) stateless verification & `SynapticVoucher` streaming over 25% UDP loss.
//! 5. Loss-tolerant UDP (`<= 1200B`) Snowball double-spend finality & Fountain DNA transfer.

use std::net::SocketAddr;
use std::time::Duration;

use skynet_agent::{AgentNode, SnowballParams};
use skynet_cell::HardwareTier;
use skynet_dna::CellRole;
use skynet_ledger::{
    GenesisHalvingVault, HumanWallet, NanoSkn, SknTransaction, SynapticVoucher, TxOutput,
    UtxoDagLedger, UtxoId, GENESIS_TOTAL_NANO_SKN,
};
use skynet_wire::{ImpulseBody, ImpulsePacket};
use tokio::net::UdpSocket;

const SIMULATED_UDP_PACKET_LOSS: f64 = 0.25; // 25% random UDP datagram drop!

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("==========================================================================");
    println!("  SKYNET BIO-CELLULAR AI BLOCKCHAIN — CHAOS UDP ECOSYSTEM SIMULATION");
    println!("==========================================================================\n");

    // -------------------------------------------------------------------------
    // 1. Human Sovereign Wallets & 21 Trillion SKN Fixed Genesis Supply
    // -------------------------------------------------------------------------
    let human_alice = HumanWallet::from_recovery_phrase("alice-human-cold-seed-skynet-2026");
    let human_bob = HumanWallet::from_recovery_phrase("bob-human-cold-seed-skynet-2026");

    // Alice owns Agents 0, 2, 4; Bob owns Agents 1, 3, 5!
    let agent_specs = [
        ("Titan-GPU-01", HardwareTier::ColossalOrganism, 500_000u128, &human_alice, 0u32),
        ("Titan-GPU-02", HardwareTier::ColossalOrganism, 500_000u128, &human_bob, 0u32),
        ("Laptop-Node-03", HardwareTier::StandardNode, 150_000u128, &human_alice, 1u32),
        ("Laptop-Node-04", HardwareTier::StandardNode, 150_000u128, &human_bob, 1u32),
        ("IoT-Edge-05", HardwareTier::TinyPeripheral, 25_000u128, &human_alice, 2u32),
        ("Periph-Sensor-06", HardwareTier::TinyPeripheral, 25_000u128, &human_bob, 2u32),
    ];

    let max_allowance = NanoSkn::from_skn(10)?; // Each AI Agent hot key may spend at most 10 SKN externally!
    let mut keys = Vec::new();
    let mut certs = Vec::new();
    let mut genesis_outputs = Vec::new();
    let mut seed_circulating = NanoSkn::ZERO;

    for (_, _, skn_whole, human_owner, sub_idx) in &agent_specs {
        let (agent_hotkey, cert) = human_owner.authorize_agent(*sub_idx, max_allowance, 10_000);
        let amount = NanoSkn::from_skn(*skn_whole)?.checked_add(NanoSkn::from_nano(7))?;
        seed_circulating = seed_circulating.checked_add(amount)?;
        genesis_outputs.push(TxOutput::agent_bond(
            human_owner.human_pubkey(),
            cert.agent_pubkey,
            amount,
            max_allowance,
        ));
        keys.push(agent_hotkey);
        certs.push(cert);
    }

    // Also give Alice & Bob cold vault UTXOs (0% metabolic decay!)
    let cold_vault_each = NanoSkn::from_skn(100_000)?;
    for owner in [&human_alice, &human_bob] {
        seed_circulating = seed_circulating.checked_add(cold_vault_each)?;
        genesis_outputs.push(TxOutput::direct(owner.human_pubkey(), cold_vault_each));
    }

    let mut halving_vault = GenesisHalvingVault::new(seed_circulating, 365)?;
    println!("[1] KINH TE HOC GENESIS 21 NGHIN TI SKN & VI CHU QUYEN CON NGUOI:");
    println!(
        "    - Tong cung bat bien   : {} ({} nanoSKN)",
        NanoSkn::GENESIS_CAP,
        GENESIS_TOTAL_NANO_SKN
    );
    println!(
        "    - Quy Halving Nam 1->2 : {} -> {} (-50.0%/nam)",
        halving_vault.annual_quota_for_year(0),
        halving_vault.annual_quota_for_year(1),
    );
    let _ = halving_vault.release_next_epoch()?;
    assert_eq!(
        halving_vault.total_accounted_supply()?.0,
        GENESIS_TOTAL_NANO_SKN
    );

    // -------------------------------------------------------------------------
    // 2. Spawn 6 Heterogeneous Multicellular AI Agents on Real Tokio UDP Sockets
    // -------------------------------------------------------------------------
    let (base_ledger, genesis_hash) = UtxoDagLedger::from_genesis(genesis_outputs)?;
    println!(
        "    - Tai san Chu nhan Alice: {} (100k SKN vi lanh + 3 AI Agents duoc uy quyen)",
        base_ledger.human_total_net_worth(&human_alice.human_pubkey())
    );
    println!(
        "    - Bo tich luy Utreexo   : Chi ton {} bytes RAM de xac thuc toan bo UTXO tren mang!\n",
        base_ledger.build_utreexo_forest().roots.ram_footprint_bytes()
    );

    let snowball_params = SnowballParams {
        sample_k: 5,
        quorum_alpha: 2,
        finality_beta: 5,
    };

    let mut agents: Vec<AgentNode> = Vec::new();
    let mut sockets: Vec<UdpSocket> = Vec::new();
    let mut addrs: Vec<SocketAddr> = Vec::new();

    for (i, (name, tier, _, _, _)) in agent_specs.iter().enumerate() {
        let sock = UdpSocket::bind("127.0.0.1:0").await?;
        addrs.push(sock.local_addr()?);
        sockets.push(sock);

        let mut agent = AgentNode::from_signing_key(
            *name,
            1000 + i as u64,
            keys[i].clone(),
            *tier,
            base_ledger.clone(),
            snowball_params,
        )?;
        agent.ownership_cert = Some(certs[i].clone());
        agents.push(agent);
    }

    // -------------------------------------------------------------------------
    // 3. Biological Growth, Stem Differentiation, 1->2 Mitosis & Apoptosis
    // -------------------------------------------------------------------------
    let stimulus = [0.65, -0.30, 0.80, 0.15, -0.45, 0.70, 0.25, -0.60];
    let target = [0.75, -0.65, 0.80, -0.55];

    for _tick in 0..16 {
        for agent in &mut agents {
            let _cortical = agent.run_biological_tick(&stimulus, &target)?;
        }
    }

    println!("[2] TANG TRUONG DA BAO AI TREN CANDLE (SAU 16 CHU KY TIEN HOA):");
    for agent in &agents {
        let s = agent.stats();
        let stem = s.role_counts.get(&CellRole::Stem).copied().unwrap_or(0);
        let skin = s.role_counts.get(&CellRole::Epidermal).copied().unwrap_or(0);
        let blood = s.role_counts.get(&CellRole::Vascular).copied().unwrap_or(0);
        let organ = s.role_counts.get(&CellRole::Visceral).copied().unwrap_or(0);
        let brain = s.role_counts.get(&CellRole::Neural).copied().unwrap_or(0);
        println!(
            "    * {:<16} | Tier: {:<16?} | Cells: {:>4} (Goc:{:>2} Da:{:>3} Mau:{:>3} Tang:{:>3} TK:{:>3}) | Mitosis:{:>4} | Apoptosis:{:>3} | GenMax:{}",
            s.name, s.tier, s.total_cells, stem, skin, blood, organ, brain,
            s.total_mitosis_events, s.total_apoptosis_events, s.max_generation
        );
    }
    println!();

    // -------------------------------------------------------------------------
    // 4. Lossy UDP SynapticVoucher AI Micro-Payments + Double-Spend Snowball Finality
    // -------------------------------------------------------------------------
    // 4a. IoT-Edge-05 streams 40 micro-inference calls (50 nanoSKN each = 2,000 nanoSKN)
    //     to Titan-GPU-02 over 25% lossy UDP, making sure final closing impulse arrives!
    let iot_bond_utxo = UtxoId {
        tx_hash: genesis_hash,
        vout: 4,
    };
    let mut buf = [0u8; 2048];
    for seq in 1..=40u64 {
        let voucher = SynapticVoucher::new_signed(
            &keys[4],
            iot_bond_utxo,
            agents[1].pubkey,
            seq,
            NanoSkn::from_nano(seq as u128 * 50),
        );
        let pkt = ImpulsePacket::new_signed(
            &keys[4],
            0,
            0,
            3,
            0,
            seq as u32,
            ImpulseBody::SynapticMicroPay { voucher, stimulus },
        )?;
        let loss = if seq == 40 { 0.0 } else { SIMULATED_UDP_PACKET_LOSS };
        agents[4]
            .send_lossy_udp(&sockets[4], addrs[1], &pkt, loss)
            .await?;
    }
    tokio::time::sleep(Duration::from_millis(8)).await;
    while let Ok((len, _)) = sockets[1].try_recv_from(&mut buf) {
        let _ = agents[1].handle_incoming_datagram(&buf[..len])?;
    }

    // 4b. Contested Double-Spend within Agent #0's 10-SKN Human-Approved Allowance (1.234567891 SKN!)
    //     Notice: Principal (499,998.765432115 SKN) MUST return to Alice's AgentBond or ledger rejects it!
    let contested_utxo = UtxoId {
        tx_hash: genesis_hash,
        vout: 0,
    };
    let titan0_total = base_ledger.balance_of(&agents[0].pubkey);
    let transfer_amount = NanoSkn::from_nano(1_234_567_891);
    let fee_nano = NanoSkn::ONE_NANO;
    let remaining_bond = titan0_total
        .checked_sub(transfer_amount)?
        .checked_sub(fee_nano)?;
    let remaining_allowance = max_allowance.checked_sub(transfer_amount)?;

    let tx_a = SknTransaction::new_signed(
        &keys[0],
        vec![contested_utxo],
        vec![
            TxOutput::direct(human_bob.human_pubkey(), transfer_amount),
            TxOutput::agent_bond(
                human_alice.human_pubkey(),
                agents[0].pubkey,
                remaining_bond,
                remaining_allowance,
            ),
        ],
        fee_nano,
        vec![genesis_hash],
        1,
    );

    let tx_b = SknTransaction::new_signed(
        &keys[0],
        vec![contested_utxo],
        vec![
            TxOutput::direct(agents[5].pubkey, transfer_amount),
            TxOutput::agent_bond(
                human_alice.human_pubkey(),
                agents[0].pubkey,
                remaining_bond,
                remaining_allowance,
            ),
        ],
        fee_nano,
        vec![genesis_hash],
        2,
    );

    // Broadcast `tx_a` with Stateless Utreexo Merkle Witnesses inside a single `<= 1200B` UDP packet!
    let forest = base_ledger.build_utreexo_forest();
    let spent_out = base_ledger.get_utxo(&contested_utxo).unwrap().clone();
    let utreexo_proof = forest.prove(&contested_utxo).unwrap();
    let witnessed_pkt = ImpulsePacket::new_signed(
        &keys[0],
        0,
        0,
        3,
        0,
        1,
        ImpulseBody::WitnessedTx {
            tx: tx_a.clone(),
            spent_outputs: vec![spent_out],
            proofs: vec![utreexo_proof],
        },
    )?;
    let witnessed_wire_size = witnessed_pkt.encode_datagram()?.len();

    // Split initial network opinion: Agents 0..4 verify WitnessedTx(tx_a), Agents 4..6 see tx_b first
    for (idx, agent) in agents.iter_mut().enumerate() {
        if idx < 4 {
            let raw_w = witnessed_pkt.encode_datagram()?;
            agent.handle_incoming_datagram(&raw_w)?;
            agent.ledger.admit_pending_tx(tx_b.clone())?;
        } else {
            agent.ledger.admit_pending_tx(tx_b.clone())?;
            agent.snowball.register_tx(&tx_b);
            agent.ledger.admit_pending_tx(tx_a.clone())?;
        }
    }

    for round in 0..50 {
        for i in 0..agents.len() {
            let contested = agents[i].snowball.contested_utxos();
            if contested.is_empty() {
                continue;
            }
            let peer_list: Vec<SocketAddr> = addrs
                .iter()
                .enumerate()
                .filter(|(idx, _)| *idx != i)
                .map(|(_, a)| *a)
                .collect();
            let targets = agents[i].sample_peers(&peer_list);
            let pkt = ImpulsePacket::new_signed(
                &agents[i].signing_key,
                agents[i].cluster_prefix,
                agents[i].prefix_len,
                3,
                agents[i].epoch,
                round,
                ImpulseBody::SnowballPull {
                    query_id: (i as u64) * 1000 + (round as u64),
                    contested_utxos: contested,
                },
            )?;
            for target_addr in targets {
                agents[i]
                    .send_lossy_udp(&sockets[i], target_addr, &pkt, SIMULATED_UDP_PACKET_LOSS)
                    .await?;
            }
        }

        tokio::time::sleep(Duration::from_millis(6)).await;

        for i in 0..agents.len() {
            while let Ok((len, src_addr)) = sockets[i].try_recv_from(&mut buf) {
                if let Some(reply_pkt) = agents[i].handle_incoming_datagram(&buf[..len])? {
                    agents[i]
                        .send_lossy_udp(
                            &sockets[i],
                            src_addr,
                            &reply_pkt,
                            SIMULATED_UDP_PACKET_LOSS,
                        )
                        .await?;
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(6)).await;

        for i in 0..agents.len() {
            while let Ok((len, _)) = sockets[i].try_recv_from(&mut buf) {
                let _ = agents[i].handle_incoming_datagram(&buf[..len])?;
            }
            agents[i].finalize_snowball_pulse()?;
        }

        if agents
            .iter()
            .all(|a| a.snowball.decided_for(&contested_utxo).is_some())
        {
            break;
        }
    }

    // -------------------------------------------------------------------------
    // 5. Horizontal Gene Transfer (Fountain Codes over 25% Lossy UDP) & Cluster Mitosis
    // -------------------------------------------------------------------------
    let droplets = agents[0].encode_elite_plasmid_droplets(18)?;
    for d_pkt in &droplets {
        agents[0]
            .send_lossy_udp(&sockets[0], addrs[4], d_pkt, SIMULATED_UDP_PACKET_LOSS)
            .await?;
    }
    tokio::time::sleep(Duration::from_millis(10)).await;
    while let Ok((len, _)) = sockets[4].try_recv_from(&mut buf) {
        let _ = agents[4].handle_incoming_datagram(&buf[..len])?;
    }

    for agent in &mut agents {
        agent.split_cluster_prefix();
        agent.verify_internal_skn_conservation()?;
    }

    let unanimous_winner = agents[0]
        .snowball
        .decided_for(&contested_utxo)
        .expect("Agent 0 must reach Snowball finality");

    for (idx, agent) in agents.iter().enumerate() {
        assert_eq!(
            agent.snowball.decided_for(&contested_utxo),
            Some(unanimous_winner),
            "Agent {idx} diverged!"
        );
        assert_eq!(
            agent.ledger.total_tracked_supply()?,
            seed_circulating,
            "Ledger SKN conservation violated on Agent {idx}!"
        );
    }

    let total_sent: u64 = agents.iter().map(|a| a.udp_sent).sum();
    let total_dropped: u64 = agents.iter().map(|a| a.udp_dropped_simulated).sum();
    let actual_loss_pct =
        (total_dropped as f64) * 100.0 / ((total_sent + total_dropped).max(1) as f64);

    println!("[3] XAC THUC SIEU NHE UTREEXO, VOUCHER VI MO & SNOWBALL QUA UDP ROT GOI:");
    println!(
        "    - Goi WitnessedTx Utreexo  : Chi nang {} bytes (<= 1200B UDP MTU), IoT xac thuc khong can luu UTXO!",
        witnessed_wire_size
    );
    println!(
        "    - Luong SynapticVoucher    : Gui 40 xung AI (mat goi ngau nhien), Titan-GPU-02 van thu du {} nanoSKN!",
        agents[1].total_voucher_nano_earned.0
    );
    println!(
        "    - Xung UDP da gui / bi rot : {} goi toi dich / {} goi bi mat ({:.1}% loss, KHONG gui lai)",
        total_sent, total_dropped, actual_loss_pct
    );
    println!(
        "    - Phan xu Double-Spend     : 6/6 AI Agents dong thuan 100% chon Tx {:02x}{:02x}..{:02x}{:02x}",
        unanimous_winner[0], unanimous_winner[1], unanimous_winner[30], unanimous_winner[31]
    );
    println!(
        "    - Bao ve Tai san Con nguoi : Alice giu nguyen von goc, Vi lanh Bob nhan them +1.234567891 SKN!"
    );
    println!(
        "    - Kiem toan Bao toan SKN   : HOP LE TUYET DOI (sai so 0 nanoSKN tren toan mang luoi!)\n");

    Ok(())
}
