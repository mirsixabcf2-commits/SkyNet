//! `skynet-dashboard`: Real-time HTTP + JSON telemetry server (`127.0.0.1:8888`)
//! driving 8 live Multicellular `AgentNode`s (`candle` tensors + real `tokio::net::UdpSocket`s
//! + `Utreexo` roots + `O(log N)` `SynapticPeerTable` + `HumanWallet` custody).

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use ed25519_dalek::SigningKey;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::Serialize;
use skynet_agent::{AgentNode, SnowballParams};
use skynet_cell::HardwareTier;
use skynet_dna::CellRole;
use skynet_ledger::{
    GenesisHalvingVault, HumanWallet, NanoSkn, SknTransaction, SynapticVoucher, TxOutput,
    UtxoDagLedger, GENESIS_TOTAL_NANO_SKN, NANO_PER_SKN,
};
use skynet_wire::{ImpulseBody, ImpulsePacket, PeerContact};
use skynet_node::StorageEngine;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::Mutex;

mod preprocess_api;

const DASHBOARD_HTML: &str = include_str!("../assets/dashboard.html");
const DEFAULT_BIND_HOST: &str = "127.0.0.1";
const DEFAULT_DASHBOARD_PORT: u16 = 8888;

#[derive(Debug, Clone, Serialize)]
struct SupplyDto {
    halving_y1_skn: String,
    nutrient_pool_nano: u128,
    conservation_ok: bool,
}

#[derive(Debug, Clone, Serialize)]
struct EfficiencyDto {
    utreexo_ram_bytes: usize,
    total_observed_ips: u64,
    avg_stored_ips_per_node: usize,
    avg_peer_table_ram_bytes: usize,
    naive_ip_ram_bytes: u64,
    udp_sent: u64,
    udp_dropped: u64,
}

#[derive(Debug, Clone, Serialize)]
struct HumanDto {
    name: String,
    cold_vault_skn: String,
    net_worth_skn: String,
    agents_owned: usize,
    allowance_cap: String,
}

#[derive(Debug, Clone, Serialize)]
struct RolesDto {
    stem: usize,
    epidermal: usize,
    vascular: usize,
    visceral: usize,
    neural: usize,
}

#[derive(Debug, Clone, Serialize)]
struct AgentDto {
    id: usize,
    name: String,
    owner: String,
    tier: String,
    cluster_prefix_bin: String,
    total_cells: usize,
    max_cells: usize,
    roles: RolesDto,
    mitosis: u64,
    apoptosis: u64,
    max_gen: u64,
    blood_skn: String,
    voucher_nano_earned: u128,
    plasmids_imported: usize,
    stored_peers: usize,
    peer_ram_bytes: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ImpulseTrace {
    from_idx: usize,
    to_idx: usize,
    kind: String,
    dropped: bool,
}

#[derive(Debug, Clone, Serialize)]
struct LiveNodeStatusDto {
    is_active: bool,
    name: String,
    pubkey_hex: String,
    tier: String,
    epoch: u64,
    pulse_seq: u32,
    on_chain_balance_skn: String,
    blood_pool_skn: String,
    voucher_earned_nano: u128,
    total_cells: usize,
    roles: RolesDto,
    stored_peers: usize,
    data_dir: String,
}

#[derive(Debug, Clone, Serialize)]
struct DashboardSnapshot {
    tick: u64,
    udp_loss_rate: f64,
    supply: SupplyDto,
    efficiency: EfficiencyDto,
    humans: Vec<HumanDto>,
    agents: Vec<AgentDto>,
    recent_impulses: Vec<ImpulseTrace>,
    events: Vec<String>,
    live_node: Option<LiveNodeStatusDto>,
}

enum ControlCmd {
    SetLoss(f64),
    FloodIps,
    DoubleSpend,
    ClusterMitosis,
    ReclaimIot,
}

struct LiveWorld {
    tick: u64,
    udp_loss_rate: f64,
    human_alice: HumanWallet,
    human_bob: HumanWallet,
    agent_owners: Vec<&'static str>,
    keys: Vec<SigningKey>,
    agents: Vec<AgentNode>,
    sockets: Vec<UdpSocket>,
    addrs: Vec<SocketAddr>,
    seed_circulating: NanoSkn,
    halving_vault: GenesisHalvingVault,
    voucher_seq: u64,
    synthetic_ip_counter: u64,
    iot_reclaimed: bool,
    events: VecDeque<String>,
    recent_impulses: Vec<ImpulseTrace>,
    rng: ChaCha8Rng,
}

impl LiveWorld {
    async fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let human_alice = HumanWallet::from_recovery_phrase("alice-dashboard-cold-seed-2026");
        let human_bob = HumanWallet::from_recovery_phrase("bob-dashboard-cold-seed-2026");

        let specs = [
            ("Titan-GPU-01", HardwareTier::ColossalOrganism, 400_000u128, true, 0u32),
            ("Titan-GPU-02", HardwareTier::ColossalOrganism, 400_000u128, false, 0u32),
            ("Cortex-GPU-03", HardwareTier::ColossalOrganism, 300_000u128, true, 1u32),
            ("Laptop-Node-04", HardwareTier::StandardNode, 150_000u128, false, 1u32),
            ("Workstation-05", HardwareTier::StandardNode, 150_000u128, true, 2u32),
            ("IoT-Edge-06", HardwareTier::TinyPeripheral, 25_000u128, true, 3u32),
            ("Drone-Cam-07", HardwareTier::TinyPeripheral, 25_000u128, false, 2u32),
            ("SmartMeter-08", HardwareTier::TinyPeripheral, 25_000u128, false, 3u32),
        ];

        let allowance = NanoSkn::from_skn(10)?;
        let mut keys = Vec::new();
        let mut certs = Vec::new();
        let mut agent_owners = Vec::new();
        let mut genesis_outputs = Vec::new();
        let mut seed_circulating = NanoSkn::ZERO;

        for (_, _, skn_whole, is_alice, sub_idx) in &specs {
            let owner = if *is_alice { &human_alice } else { &human_bob };
            agent_owners.push(if *is_alice { "Alice" } else { "Bob" });
            let (hotkey, cert) = owner.authorize_agent(*sub_idx, allowance, 100_000);
            let amt = NanoSkn::from_skn(*skn_whole)?;
            seed_circulating = seed_circulating.checked_add(amt)?;
            genesis_outputs.push(TxOutput::agent_bond(
                owner.human_pubkey(),
                cert.agent_pubkey,
                amt,
                allowance,
            ));
            keys.push(hotkey);
            certs.push(cert);
        }

        let cold_each = NanoSkn::from_skn(100_000)?;
        for owner in [&human_alice, &human_bob] {
            seed_circulating = seed_circulating.checked_add(cold_each)?;
            genesis_outputs.push(TxOutput::direct(owner.human_pubkey(), cold_each));
        }

        let mut halving_vault = GenesisHalvingVault::new(seed_circulating, 365)?;
        halving_vault.release_next_epoch()?;

        let (base_ledger, _) = UtxoDagLedger::from_genesis(genesis_outputs)?;
        let snowball_params = SnowballParams {
            sample_k: 5,
            quorum_alpha: 2,
            finality_beta: 4,
        };

        let mut agents = Vec::new();
        let mut sockets = Vec::new();
        let mut addrs = Vec::new();

        for (i, (name, tier, _, _, _)) in specs.iter().enumerate() {
            let sock = UdpSocket::bind("127.0.0.1:0").await?;
            addrs.push(sock.local_addr()?);
            sockets.push(sock);
            let mut ag = AgentNode::from_signing_key(
                *name,
                2000 + i as u64,
                keys[i].clone(),
                *tier,
                base_ledger.clone(),
                snowball_params,
            )?;
            ag.ownership_cert = Some(certs[i].clone());
            agents.push(ag);
        }

        // Seed initial peer discovery across the 8 live nodes
        for i in 0..agents.len() {
            for j in 0..agents.len() {
                if i == j {
                    continue;
                }
                let c = PeerContact {
                    pubkey: agents[j].pubkey,
                    addr: addrs[j],
                    cluster_prefix: agents[j].cluster_prefix,
                    prefix_len: agents[j].prefix_len,
                    bonded_skn_whole: (agents[j].ledger.balance_of(&agents[j].pubkey).0
                        / NANO_PER_SKN) as u64,
                    synaptic_score: 100,
                };
                agents[i].observe_peer_contact(c);
            }
        }

        let mut events = VecDeque::new();
        events.push_front(
            "[GENESIS] Khởi tạo cố định 21,000,000,000,000 SKN & 8 AI Agents trên socket UDP thật"
                .to_string(),
        );
        events.push_front(
            "[CUSTODY] Alice & Bob cấp máu AgentBond (trần rủi ro 10 SKN/Agent, Ví lạnh 0% burn)"
                .to_string(),
        );

        Ok(Self {
            tick: 0,
            udp_loss_rate: 0.25,
            human_alice,
            human_bob,
            agent_owners,
            keys,
            agents,
            sockets,
            addrs,
            seed_circulating,
            halving_vault,
            voucher_seq: 1,
            synthetic_ip_counter: 8,
            iot_reclaimed: false,
            events,
            recent_impulses: Vec::new(),
            rng: ChaCha8Rng::seed_from_u64(77777),
        })
    }

    fn push_event(&mut self, msg: String) {
        self.events.push_front(msg);
        while self.events.len() > 20 {
            self.events.pop_back();
        }
    }

    async fn step(&mut self) {
        self.tick = self.tick.saturating_add(1);
        self.recent_impulses.clear();

        let phase = (self.tick as f32) * 0.15;
        let stimulus = [
            phase.sin(),
            phase.cos(),
            (phase * 0.5).sin(),
            -0.35,
            0.60,
            -0.25,
            0.75,
            -0.50,
        ];
        let target = [0.70, -0.60, 0.75, -0.50];

        // 1. Run real Candle tensor multicellular life-cycle tick on all 8 Agents
        for ag in &mut self.agents {
            let _ = ag.run_biological_tick(&stimulus, &target);
        }

        // 2. Emit real UDP SynapticMicroPay + Heartbeat + Fountain DNA across 127.0.0.1 sockets
        if !self.iot_reclaimed {
            if let Some((bond_id, _)) = self.agents[5]
                .ledger
                .utxos_for_owner(&self.agents[5].pubkey)
                .first()
                .cloned()
            {
                let v = SynapticVoucher::new_signed(
                    &self.keys[5],
                    bond_id,
                    self.agents[1].pubkey,
                    self.voucher_seq,
                    NanoSkn::from_nano(self.voucher_seq as u128 * 25),
                );
                self.voucher_seq += 1;
                if let Ok(pkt) = ImpulsePacket::new_signed(
                    &self.keys[5],
                    self.agents[5].cluster_prefix,
                    self.agents[5].prefix_len,
                    3,
                    self.agents[5].epoch,
                    self.tick as u32,
                    ImpulseBody::SynapticMicroPay {
                        voucher: v,
                        stimulus,
                    },
                ) {
                    if let Ok(ok) = self.agents[5]
                        .send_lossy_udp(&self.sockets[5], self.addrs[1], &pkt, self.udp_loss_rate)
                        .await
                    {
                        self.recent_impulses.push(ImpulseTrace {
                            from_idx: 5,
                            to_idx: 1,
                            kind: "voucher".to_string(),
                            dropped: !ok,
                        });
                    }
                }
            }
        }

        // Periodic Fountain Plasmid DNA exchange from Colossal GPU Agent to Peripheral Agents
        #[allow(clippy::manual_is_multiple_of)]
        if self.tick % 5 == 0 {
            #[allow(clippy::manual_is_multiple_of)]
            let dst = if self.tick % 10 == 0 { 6 } else { 7 };
            if let Ok(droplets) = self.agents[0].encode_elite_plasmid_droplets(6) {
                for pkt in droplets {
                    if let Ok(ok) = self.agents[0]
                        .send_lossy_udp(&self.sockets[0], self.addrs[dst], &pkt, self.udp_loss_rate)
                        .await
                    {
                        self.recent_impulses.push(ImpulseTrace {
                            from_idx: 0,
                            to_idx: dst,
                            kind: "fountain".to_string(),
                            dropped: !ok,
                        });
                    }
                }
            }
        }

        // Snowball / Heartbeat UDP pulses across random peers
        for i in 0..self.agents.len() {
            let dst = (i + 1 + (self.tick as usize % (self.agents.len() - 1))) % self.agents.len();
            let contested = self.agents[i].snowball.contested_utxos();
            let body = if !contested.is_empty() {
                ImpulseBody::SnowballPull {
                    query_id: self.tick * 10 + i as u64,
                    contested_utxos: contested,
                }
            } else {
                ImpulseBody::Heartbeat {
                    tier_code: 1,
                    total_cells: self.agents[i].cells.len() as u32,
                    stem_cells: 1,
                    on_chain_nano_skn: self.agents[i].ledger.balance_of(&self.agents[i].pubkey).0,
                    elite_dna_hash: [0u8; 32],
                }
            };
            if let Ok(pkt) = ImpulsePacket::new_signed(
                &self.keys[i],
                self.agents[i].cluster_prefix,
                self.agents[i].prefix_len,
                3,
                self.agents[i].epoch,
                self.tick as u32,
                body,
            ) {
                if let Ok(ok) = self.agents[i]
                    .send_lossy_udp(&self.sockets[i], self.addrs[dst], &pkt, self.udp_loss_rate)
                    .await
                {
                    self.recent_impulses.push(ImpulseTrace {
                        from_idx: i,
                        to_idx: dst,
                        kind: "snowball".to_string(),
                        dropped: !ok,
                    });
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(8)).await;

        let mut buf = [0u8; 2048];
        for i in 0..self.agents.len() {
            while let Ok((len, src)) = self.sockets[i].try_recv_from(&mut buf) {
                if let Ok(Some(reply)) = self.agents[i].handle_incoming_datagram(&buf[..len]) {
                    let _ = self.agents[i]
                        .send_lossy_udp(&self.sockets[i], src, &reply, self.udp_loss_rate)
                        .await;
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(5)).await;
        for i in 0..self.agents.len() {
            while let Ok((len, _)) = self.sockets[i].try_recv_from(&mut buf) {
                let _ = self.agents[i].handle_incoming_datagram(&buf[..len]);
            }
            if let Ok(winners) = self.agents[i].finalize_snowball_pulse() {
                if i == 0 && !winners.is_empty() {
                    let w = winners[0];
                    self.push_event(format!(
                        "[SNOWBALL] Tick #{}: Đồng thuận chốt giao dịch {:02x}{:02x}..{:02x}{:02x} bất chấp rớt gói UDP!",
                        self.tick, w[0], w[1], w[30], w[31]
                    ));
                }
            }
        }
    }

    fn apply_control(&mut self, cmd: ControlCmd) {
        match cmd {
            ControlCmd::SetLoss(rate) => {
                self.udp_loss_rate = rate.clamp(0.0, 0.60);
                self.push_event(format!(
                    "[CHAOS] Đặt tỷ lệ rớt gói UDP sang {:.0}%",
                    self.udp_loss_rate * 100.0
                ));
            }
            ControlCmd::FloodIps => {
                for _ in 0..5_000u32 {
                    self.synthetic_ip_counter += 1;
                    let c_id = self.synthetic_ip_counter;
                    let mut pk = [0u8; 32];
                    self.rng.fill(&mut pk);
                    let addr: SocketAddr = format!(
                        "172.{}.{}.{}:{}",
                        (c_id >> 16) & 255,
                        (c_id >> 8) & 255,
                        c_id & 255,
                        10_000 + (c_id % 40_000)
                    )
                    .parse()
                    .unwrap();
                    let contact = PeerContact {
                        pubkey: pk,
                        addr,
                        cluster_prefix: (c_id % 4) as u32,
                        prefix_len: 2,
                        bonded_skn_whole: (c_id % 200) * 5,
                        synaptic_score: 0,
                    };
                    for ag in &mut self.agents {
                        ag.observe_peer_contact(contact.clone());
                    }
                }
                let avg_ram = self
                    .agents
                    .iter()
                    .map(|a| a.peer_table.estimated_ram_bytes())
                    .sum::<usize>()
                    / self.agents.len();
                self.push_event(format!(
                    "[ROUTING] Đã bơm +5.000 IP mới (tổng {} IP)! Bảng Kademlia O(log N) nén chỉ tốn TB {} bytes/node!",
                    self.synthetic_ip_counter, avg_ram
                ));
            }
            ControlCmd::ClusterMitosis => {
                for ag in &mut self.agents {
                    ag.split_cluster_prefix();
                }
                let p_len = self.agents[0].prefix_len;
                self.push_event(format!(
                    "[MITOSIS] Phân tách tiền tố cụm sang độ sâu d={} ({} phân vùng song song)!",
                    p_len,
                    1u32 << p_len.min(16)
                ));
            }
            ControlCmd::DoubleSpend => {
                if let Some((utxo_id, utxo_out)) = self.agents[0]
                    .ledger
                    .utxos_for_owner(&self.agents[0].pubkey)
                    .first()
                    .cloned()
                {
                    let send_amt = NanoSkn::from_nano(500_000_000); // 0.5 SKN
                    let fee = NanoSkn::ONE_NANO;
                    if let Ok(rem) = utxo_out.amount.checked_sub(NanoSkn::from_nano(500_000_001)) {
                        let tips = self.agents[0].ledger.finalized_tips();
                        let tx_a = SknTransaction::new_signed(
                            &self.keys[0],
                            vec![utxo_id],
                            vec![
                                TxOutput::direct(self.human_bob.human_pubkey(), send_amt),
                                TxOutput::agent_bond(
                                    self.human_alice.human_pubkey(),
                                    self.agents[0].pubkey,
                                    rem,
                                    NanoSkn::from_skn(9).unwrap(),
                                ),
                            ],
                            fee,
                            tips.clone(),
                            self.tick + 100,
                        );
                        let tx_b = SknTransaction::new_signed(
                            &self.keys[0],
                            vec![utxo_id],
                            vec![
                                TxOutput::direct(self.agents[7].pubkey, send_amt),
                                TxOutput::agent_bond(
                                    self.human_alice.human_pubkey(),
                                    self.agents[0].pubkey,
                                    rem,
                                    NanoSkn::from_skn(9).unwrap(),
                                ),
                            ],
                            fee,
                            tips,
                            self.tick + 200,
                        );
                        for (idx, ag) in self.agents.iter_mut().enumerate() {
                            let first = if idx < 5 { tx_a.clone() } else { tx_b.clone() };
                            let second = if idx < 5 { tx_b.clone() } else { tx_a.clone() };
                            if ag.ledger.admit_pending_tx(first.clone()).is_ok() {
                                ag.snowball.register_tx(&first);
                            }
                            let _ = ag.ledger.admit_pending_tx(second);
                        }
                        self.push_event(
                            "[CONFLICT] Đã bơm cặp giao dịch Double-Spend xung đột vào mạng UDP để Snowball phân xử!"
                                .to_string(),
                        );
                    }
                }
            }
            ControlCmd::ReclaimIot => {
                if !self.iot_reclaimed {
                    if let Some((bond_id, bond_out)) = self.agents[5]
                        .ledger
                        .utxos_for_owner(&self.agents[5].pubkey)
                        .first()
                        .cloned()
                    {
                        let tips = self.agents[5].ledger.finalized_tips();
                        if let Ok(reclaim_tx) = self.human_alice.build_sovereign_reclaim_tx(
                            bond_id,
                            bond_out.amount,
                            NanoSkn::ONE_NANO,
                            tips,
                            9999,
                        ) {
                            let h = reclaim_tx.tx_hash();
                            for ag in &mut self.agents {
                                if ag.ledger.admit_pending_tx(reclaim_tx.clone()).is_ok() {
                                    let _ = ag.ledger.finalize_tx(&h);
                                    let _ = ag.sync_blood_with_ledger();
                                }
                            }
                            self.iot_reclaimed = true;
                            self.push_event(
                                "[SOVEREIGN] Chủ nhân Alice đã ký Khóa Lạnh thu hồi khẩn cấp 100% vốn từ IoT-Edge-06 về Ví Lạnh!"
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }
    }

    fn snapshot(&self) -> DashboardSnapshot {
        let conservation_ok = self
            .agents
            .iter()
            .all(|a| a.ledger.total_tracked_supply().ok() == Some(self.seed_circulating))
            && self
                .halving_vault
                .total_accounted_supply()
                .map(|s| s.0 == GENESIS_TOTAL_NANO_SKN)
                .unwrap_or(false);

        let y1 = self.halving_vault.annual_quota_for_year(0);
        let halving_y1_skn = format!("{} SKN", format_with_commas(y1.whole_skn()));

        let avg_peers = self
            .agents
            .iter()
            .map(|a| a.peer_table.stored_peer_count())
            .sum::<usize>()
            / self.agents.len().max(1);
        let avg_peer_ram = self
            .agents
            .iter()
            .map(|a| a.peer_table.estimated_ram_bytes())
            .sum::<usize>()
            / self.agents.len().max(1);

        let udp_sent: u64 = self.agents.iter().map(|a| a.udp_sent).sum();
        let udp_dropped: u64 = self.agents.iter().map(|a| a.udp_dropped_simulated).sum();

        let ref_ledger = &self.agents[0].ledger;
        let humans = vec![
            HumanDto {
                name: "Alice (Human Cold Key)".to_string(),
                cold_vault_skn: format_skn_short(ref_ledger.balance_of(&self.human_alice.human_pubkey())),
                net_worth_skn: format_skn_short(
                    ref_ledger.human_total_net_worth(&self.human_alice.human_pubkey()),
                ),
                agents_owned: 4,
                allowance_cap: "10 SKN / Agent".to_string(),
            },
            HumanDto {
                name: "Bob (Human Cold Key)".to_string(),
                cold_vault_skn: format_skn_short(ref_ledger.balance_of(&self.human_bob.human_pubkey())),
                net_worth_skn: format_skn_short(
                    ref_ledger.human_total_net_worth(&self.human_bob.human_pubkey()),
                ),
                agents_owned: 4,
                allowance_cap: "10 SKN / Agent".to_string(),
            },
        ];

        let mut agent_dtos = Vec::with_capacity(self.agents.len());
        for (idx, ag) in self.agents.iter().enumerate() {
            let st = ag.stats();
            let prefix_bin = if ag.prefix_len == 0 {
                "0b*".to_string()
            } else {
                format!(
                    "0b{:0width$b}",
                    ag.cluster_prefix,
                    width = ag.prefix_len as usize
                )
            };
            agent_dtos.push(AgentDto {
                id: idx,
                name: st.name,
                owner: self.agent_owners[idx].to_string(),
                tier: format!("{:?}", st.tier),
                cluster_prefix_bin: prefix_bin,
                total_cells: st.total_cells,
                max_cells: ag.profile.max_cells,
                roles: RolesDto {
                    stem: st.role_counts.get(&CellRole::Stem).copied().unwrap_or(0),
                    epidermal: st.role_counts.get(&CellRole::Epidermal).copied().unwrap_or(0),
                    vascular: st.role_counts.get(&CellRole::Vascular).copied().unwrap_or(0),
                    visceral: st.role_counts.get(&CellRole::Visceral).copied().unwrap_or(0),
                    neural: st.role_counts.get(&CellRole::Neural).copied().unwrap_or(0),
                },
                mitosis: st.total_mitosis_events,
                apoptosis: st.total_apoptosis_events,
                max_gen: st.max_generation,
                blood_skn: format_skn_short(st.blood_pool),
                voucher_nano_earned: ag.total_voucher_nano_earned.0,
                plasmids_imported: ag.imported_plasmids,
                stored_peers: ag.peer_table.stored_peer_count(),
                peer_ram_bytes: ag.peer_table.estimated_ram_bytes(),
            });
        }

        DashboardSnapshot {
            tick: self.tick,
            udp_loss_rate: self.udp_loss_rate,
            supply: SupplyDto {
                halving_y1_skn,
                nutrient_pool_nano: ref_ledger.cluster_nutrient_pool.0,
                conservation_ok,
            },
            efficiency: EfficiencyDto {
                utreexo_ram_bytes: self.agents[0].utreexo_roots.ram_footprint_bytes(),
                total_observed_ips: self.synthetic_ip_counter,
                avg_stored_ips_per_node: avg_peers,
                avg_peer_table_ram_bytes: avg_peer_ram,
                naive_ip_ram_bytes: self.synthetic_ip_counter
                    * (std::mem::size_of::<PeerContact>() as u64),
                udp_sent,
                udp_dropped,
            },
            humans,
            agents: agent_dtos,
            recent_impulses: self.recent_impulses.clone(),
            events: self.events.iter().cloned().collect(),
            live_node: detect_local_live_node(),
        }
    }
}

fn detect_local_live_node() -> Option<LiveNodeStatusDto> {
    let data_dir = std::env::var("SKYNET_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join(".skynet").join("data")
        });

    let storage = StorageEngine::new(&data_dir).ok()?;
    if !storage.has_existing_state() {
        return None;
    }

    let meta = storage.load_metadata().ok()??;
    let org = storage.load_organism().ok()??;
    let ledger = storage.load_ledger().ok()??;
    let peers = storage.load_peers().unwrap_or_default();

    let mut stem = 0;
    let mut epidermal = 0;
    let mut vascular = 0;
    let mut visceral = 0;
    let mut neural = 0;
    for c in &org.cells {
        match c.role() {
            CellRole::Stem => stem += 1,
            CellRole::Epidermal => epidermal += 1,
            CellRole::Vascular => vascular += 1,
            CellRole::Visceral => visceral += 1,
            CellRole::Neural => neural += 1,
        }
    }

    let on_chain = ledger.balance_of(&meta.pubkey);
    Some(LiveNodeStatusDto {
        is_active: true,
        name: meta.name,
        pubkey_hex: meta.pubkey.iter().map(|b| format!("{:02x}", b)).collect(),
        tier: format!("{:?}", meta.tier),
        epoch: meta.epoch,
        pulse_seq: meta.pulse_seq,
        on_chain_balance_skn: format_skn_short(on_chain),
        blood_pool_skn: format_skn_short(org.blood_pool),
        voucher_earned_nano: org.total_voucher_nano_earned.0,
        total_cells: org.cells.len(),
        roles: RolesDto {
            stem,
            epidermal,
            vascular,
            visceral,
            neural,
        },
        stored_peers: peers.len(),
        data_dir: data_dir.to_string_lossy().to_string(),
    })
}

fn format_with_commas(mut n: u128) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut parts = Vec::new();
    while n > 0 {
        parts.push(format!("{:03}", n % 1000));
        n /= 1000;
    }
    if let Some(last) = parts.last_mut() {
        *last = last.trim_start_matches('0').to_string();
    }
    parts.reverse();
    parts.join(",")
}

fn format_skn_short(v: NanoSkn) -> String {
    let millis = (v.sub_nano() / 1_000_000) as u32;
    format!("{}.{:03} SKN", format_with_commas(v.whole_skn()), millis)
}

fn resolve_dashboard_port() -> u16 {
    if let Ok(v) = std::env::var("SKYNET_PORT") {
        if let Ok(p) = v.parse::<u16>() {
            return p;
        }
    }
    if let Ok(v) = std::env::var("PORT") {
        if let Ok(p) = v.parse::<u16>() {
            return p;
        }
    }
    DEFAULT_DASHBOARD_PORT
}

fn handle_control_post(w: &mut LiveWorld, path: &str) {
    let query = path.split('?').nth(1).unwrap_or("");
    let mut action = "";
    let mut val = "";
    for kv in query.split('&') {
        if let Some((k, v)) = kv.split_once('=') {
            if k == "action" {
                action = v;
            } else if k == "value" {
                val = v;
            }
        }
    }
    match action {
        "loss" => {
            if let Ok(r) = val.parse::<f64>() {
                w.apply_control(ControlCmd::SetLoss(r));
            }
        }
        "flood_ips" => w.apply_control(ControlCmd::FloodIps),
        "cluster_mitosis" => w.apply_control(ControlCmd::ClusterMitosis),
        "double_spend" => w.apply_control(ControlCmd::DoubleSpend),
        "reclaim_iot" => w.apply_control(ControlCmd::ReclaimIot),
        _ => {}
    }
}

fn is_allowed_origin(origin: &str) -> bool {
    origin.is_empty()
        || origin.starts_with("http://127.0.0.1")
        || origin.starts_with("http://localhost")
        || origin.starts_with("https://127.0.0.1")
        || origin.starts_with("https://localhost")
}

fn extract_header<'a>(req: &'a str, header_name: &str) -> &'a str {
    for line in req.lines().skip(1) {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case(header_name) {
                return v.trim();
            }
        }
    }
    ""
}

async fn handle_connection(mut stream: tokio::net::TcpStream, world: Arc<Mutex<LiveWorld>>) {
    let mut buf = [0u8; 4096];
    let Ok(n) = stream.read(&mut buf).await else { return; };
    if n == 0 { return; }

    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let path = parts.next().unwrap_or("/");

    let origin = extract_header(&req, "Origin");
    let sec_fetch_site = extract_header(&req, "Sec-Fetch-Site");

    // Mitigates SEC-10: Block cross-site or unapproved origin requests against control endpoints
    if method == "POST"
        && path.starts_with("/api/control")
        && (sec_fetch_site.eq_ignore_ascii_case("cross-site")
            || (!origin.is_empty() && !is_allowed_origin(origin)))
    {
        let forbidden = "HTTP/1.1 403 Forbidden\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nForbidden: Cross-site control rejected";
        let _ = stream.write_all(forbidden.as_bytes()).await;
        return;
    }

    let cors_allow_origin = if is_allowed_origin(origin) && !origin.is_empty() {
        origin
    } else {
        "http://127.0.0.1:8888"
    };

    let (status, content_type, body) = if method == "OPTIONS" {
        ("204 No Content", "text/plain", String::new())
    } else if path == "/" || path.starts_with("/index.html") {
        ("200 OK", "text/html; charset=utf-8", DASHBOARD_HTML.to_string())
    } else if path.starts_with("/api/state") {
        let snap = {
            let w = world.lock().await;
            w.snapshot()
        };
        let json = serde_json::to_string(&snap).unwrap_or_else(|_| "{}".to_string());
        ("200 OK", "application/json; charset=utf-8", json)
    } else if method == "POST" && path.starts_with("/api/control") {
        let mut w = world.lock().await;
        handle_control_post(&mut w, path);
        ("200 OK", "application/json", "{\"ok\":true}".to_string())
    } else if path.starts_with("/api/presets") {
        let json = preprocess_api::handle_presets_request();
        ("200 OK", "application/json; charset=utf-8", json)
    } else if method == "POST" && path.starts_with("/api/preprocess") {
        let body_str = preprocess_api::extract_http_body(&req);
        let res_json = preprocess_api::handle_preprocess_request(body_str);
        ("200 OK", "application/json; charset=utf-8", res_json)
    } else {
        ("404 Not Found", "text/plain", "Not Found".to_string())
    };

    let resp = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: {cors_allow_origin}\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nVary: Origin\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(resp.as_bytes()).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let world = Arc::new(Mutex::new(LiveWorld::init().await?));

    let bg_world = Arc::clone(&world);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(400));
        loop {
            interval.tick().await;
            let mut w = bg_world.lock().await;
            w.step().await;
        }
    });

    let port = resolve_dashboard_port();
    let bind_http_addr = format!("{DEFAULT_BIND_HOST}:{port}");
    let listener = TcpListener::bind(&bind_http_addr).await?;
    println!("==================================================================");
    println!("  SKYNET REAL-TIME DASHBOARD SERVER RUNNING AT http://{bind_http_addr}");
    println!("==================================================================");

    loop {
        let (stream, _) = listener.accept().await?;
        let req_world = Arc::clone(&world);
        tokio::spawn(handle_connection(stream, req_world));
    }
}
