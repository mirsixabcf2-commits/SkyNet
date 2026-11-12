//! `live_world`: 8-node multicellular `AgentNode` UDP cluster simulation and
//! `DashboardSnapshot` synthesis for `skynet-dashboard`.

use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::net::SocketAddr;
use std::time::Duration;

use ed25519_dalek::SigningKey;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use skynet_agent::{AgentError, AgentNode, SnowballParams};
use skynet_cell::HardwareTier;
use skynet_ledger::{
    AgentOwnershipCertificate, GenesisHalvingVault, HumanWallet, LedgerError, NanoSkn,
    SynapticVoucher, TxOutput, UtxoDagLedger, GENESIS_TOTAL_NANO_SKN, NANO_PER_SKN,
};
use skynet_wire::{ImpulseBody, ImpulsePacket, PeerContact, WireError};
use tokio::net::UdpSocket;

use crate::dto::{build_agents_dto, build_efficiency_dto, build_humans_dto, format_with_commas, DashboardSnapshot, ImpulseTrace, SupplyDto, WorkerStudioTelemetryDto};
use crate::world_actions::detect_local_live_node;

type NodeSpec = (&'static str, HardwareTier, u128, bool, u32);
type GenesisClusterSeed = (Vec<SigningKey>, Vec<AgentOwnershipCertificate>, Vec<&'static str>, Vec<TxOutput>, NanoSkn);

const CLUSTER_SPECS: [NodeSpec; 8] = [
    ("Titan-GPU-01", HardwareTier::ColossalOrganism, 400_000, true, 0),
    ("Titan-GPU-02", HardwareTier::ColossalOrganism, 400_000, false, 0),
    ("Cortex-GPU-03", HardwareTier::ColossalOrganism, 300_000, true, 1),
    ("Laptop-Node-04", HardwareTier::StandardNode, 150_000, false, 1),
    ("Workstation-05", HardwareTier::StandardNode, 150_000, true, 2),
    ("IoT-Edge-06", HardwareTier::TinyPeripheral, 25_000, true, 3),
    ("Drone-Cam-07", HardwareTier::TinyPeripheral, 25_000, false, 2),
    ("SmartMeter-08", HardwareTier::TinyPeripheral, 25_000, false, 3),
];

#[derive(Debug)]
pub enum DashboardError { Io(std::io::Error), Ledger(LedgerError), Wire(WireError), Agent(AgentError) }
impl fmt::Display for DashboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self { Self::Io(e) => write!(f, "I/O: {e}"), Self::Ledger(e) => write!(f, "ledger: {e}"), Self::Wire(e) => write!(f, "wire: {e}"), Self::Agent(e) => write!(f, "agent: {e}") }
    }
}
impl std::error::Error for DashboardError {}
impl From<std::io::Error> for DashboardError { fn from(e: std::io::Error) -> Self { Self::Io(e) } }
impl From<LedgerError> for DashboardError { fn from(e: LedgerError) -> Self { Self::Ledger(e) } }
impl From<WireError> for DashboardError { fn from(e: WireError) -> Self { Self::Wire(e) } }
impl From<AgentError> for DashboardError { fn from(e: AgentError) -> Self { Self::Agent(e) } }

pub struct LiveWorld {
    pub tick: u64, pub udp_loss_rate: f64,
    pub(crate) human_alice: HumanWallet, pub(crate) human_bob: HumanWallet,
    pub(crate) agent_owners: Vec<&'static str>, pub(crate) keys: Vec<SigningKey>,
    pub(crate) agents: Vec<AgentNode>, pub(crate) sockets: Vec<UdpSocket>, pub(crate) addrs: Vec<SocketAddr>,
    pub(crate) seed_circulating: NanoSkn, pub(crate) halving_vault: GenesisHalvingVault,
    pub(crate) voucher_seq: u64, pub(crate) synthetic_ip_counter: u64, pub(crate) iot_reclaimed: bool,
    pub(crate) events: VecDeque<String>, pub(crate) recent_impulses: Vec<ImpulseTrace>, pub(crate) rng: ChaCha8Rng,
}

impl LiveWorld {
    pub async fn init() -> Result<Self, DashboardError> {
        let human_alice = HumanWallet::from_recovery_phrase("alice-dashboard-cold-seed-2026");
        let human_bob = HumanWallet::from_recovery_phrase("bob-dashboard-cold-seed-2026");
        let (keys, certs, agent_owners, genesis_outputs, seed_circulating) = seed_genesis_allocations(&human_alice, &human_bob)?;
        let mut halving_vault = GenesisHalvingVault::new(seed_circulating, 365)?;
        halving_vault.release_next_epoch()?;
        let (base_ledger, _) = UtxoDagLedger::from_genesis(genesis_outputs)?;
        let (agents, sockets, addrs) = spawn_cluster_agents(&CLUSTER_SPECS, &keys, &certs, &base_ledger).await?;
        let events = VecDeque::from([
            "[CUSTODY] Alice & Bob cấp máu AgentBond (trần rủi ro 10 SKN/Agent, Ví lạnh 0% burn)".into(),
            "[GENESIS] Khởi tạo cố định 21,000,000,000,000 SKN & 8 AI Agents trên socket UDP thật".into(),
        ]);
        Ok(Self {
            tick: 0, udp_loss_rate: 0.25, human_alice, human_bob, agent_owners, keys, agents, sockets, addrs,
            seed_circulating, halving_vault, voucher_seq: 1, synthetic_ip_counter: 8, iot_reclaimed: false,
            events, recent_impulses: Vec::new(), rng: ChaCha8Rng::seed_from_u64(77_777),
        })
    }

    pub fn push_event(&mut self, msg: String) {
        self.events.push_front(msg);
        while self.events.len() > 20 { self.events.pop_back(); }
    }

    pub async fn step(&mut self) {
        self.tick = self.tick.saturating_add(1);
        self.recent_impulses.clear();
        let p = (self.tick as f32) * 0.15;
        let stimulus = [p.sin(), p.cos(), (p * 0.5).sin(), -0.35, 0.60, -0.25, 0.75, -0.50];
        let target = [0.70, -0.60, 0.75, -0.50];
        for ag in &mut self.agents { let _ = ag.run_biological_tick(&stimulus, &target); }
        self.try_emit_iot_voucher(&stimulus).await;
        self.try_emit_fountain_droplets().await;
        self.emit_snowball_or_heartbeat().await;
        tokio::time::sleep(Duration::from_millis(8)).await;
        self.drain_and_finalize_pulses().await;
    }

    pub fn snapshot(&self) -> DashboardSnapshot {
        self.snapshot_with_studio(&WorkerStudioTelemetryDto::default(), &[])
    }

    pub fn snapshot_with_studio(&self, studio: &WorkerStudioTelemetryDto, sweep_log: &[String]) -> DashboardSnapshot {
        let conservation_ok = self.agents.iter().all(|a| a.ledger.total_tracked_supply().ok() == Some(self.seed_circulating))
            && self.halving_vault.total_accounted_supply().map(|s| s.0 == GENESIS_TOTAL_NANO_SKN).unwrap_or(false);
        let y1 = self.halving_vault.annual_quota_for_year(0);
        let ref_ledger = &self.agents[0].ledger;
        DashboardSnapshot {
            tick: self.tick, udp_loss_rate: self.udp_loss_rate,
            supply: SupplyDto { halving_y1_skn: format!("{} SKN", format_with_commas(y1.whole_skn())), nutrient_pool_nano: ref_ledger.cluster_nutrient_pool.0, conservation_ok },
            efficiency: build_efficiency_dto(&self.agents, self.synthetic_ip_counter),
            humans: build_humans_dto(ref_ledger, &self.human_alice, &self.human_bob),
            agents: build_agents_dto(&self.agents, &self.agent_owners),
            recent_impulses: self.recent_impulses.clone(), events: self.events.iter().cloned().collect(),
            live_node: detect_local_live_node(), worker_studio: studio.clone(), yield_sweep_log: sweep_log.to_vec(),
        }
    }

    async fn try_emit_iot_voucher(&mut self, stimulus: &[f32; 8]) {
        if self.iot_reclaimed { return; }
        let Some((bond_id, _)) = self.agents[5].ledger.utxos_for_owner(&self.agents[5].pubkey).first().cloned() else { return };
        let v = SynapticVoucher::new_signed(&self.keys[5], bond_id, self.agents[1].pubkey, self.voucher_seq, NanoSkn::from_nano(u128::from(self.voucher_seq) * 25));
        self.voucher_seq += 1;
        let Ok(pkt) = ImpulsePacket::new_signed(&self.keys[5], self.agents[5].cluster_prefix, self.agents[5].prefix_len, 3, self.agents[5].epoch, self.tick as u32, ImpulseBody::SynapticMicroPay { voucher: v, stimulus: *stimulus }) else { return };
        if let Ok(ok) = self.agents[5].send_lossy_udp(&self.sockets[5], self.addrs[1], &pkt, self.udp_loss_rate).await {
            self.recent_impulses.push(ImpulseTrace { from_idx: 5, to_idx: 1, kind: "voucher".into(), dropped: !ok });
        }
    }

    async fn try_emit_fountain_droplets(&mut self) {
        if self.tick.rem_euclid(5) != 0 { return; }
        let dst = if self.tick.rem_euclid(10) == 0 { 6 } else { 7 };
        let Ok(droplets) = self.agents[0].encode_elite_plasmid_droplets(6) else { return };
        for pkt in droplets {
            if let Ok(ok) = self.agents[0].send_lossy_udp(&self.sockets[0], self.addrs[dst], &pkt, self.udp_loss_rate).await {
                self.recent_impulses.push(ImpulseTrace { from_idx: 0, to_idx: dst, kind: "fountain".into(), dropped: !ok });
            }
        }
    }

    async fn emit_snowball_or_heartbeat(&mut self) {
        for i in 0..self.agents.len() {
            let dst = (i + 1 + (self.tick as usize % (self.agents.len() - 1))) % self.agents.len();
            let dst_pk = self.agents[dst].pubkey;
            let contested = self.agents[i].snowball.contested_utxos();
            let body = if !contested.is_empty() {
                let qid = self.tick * 10 + i as u64;
                self.agents[i].initiate_snowball_query(qid, HashSet::from([dst_pk]), contested.clone());
                ImpulseBody::SnowballPull { query_id: qid, contested_utxos: contested }
            } else {
                ImpulseBody::Heartbeat { tier_code: 1, total_cells: self.agents[i].cells.len() as u32, stem_cells: 1, on_chain_nano_skn: self.agents[i].ledger.balance_of(&self.agents[i].pubkey).0, elite_dna_hash: [0u8; 32] }
            };
            let Ok(pkt) = ImpulsePacket::new_signed(&self.keys[i], self.agents[i].cluster_prefix, self.agents[i].prefix_len, 3, self.agents[i].epoch, self.tick as u32, body) else { continue };
            if let Ok(ok) = self.agents[i].send_lossy_udp(&self.sockets[i], self.addrs[dst], &pkt, self.udp_loss_rate).await {
                self.recent_impulses.push(ImpulseTrace { from_idx: i, to_idx: dst, kind: "snowball".into(), dropped: !ok });
            }
        }
    }

    async fn drain_and_finalize_pulses(&mut self) {
        let mut buf = [0u8; 2048];
        for i in 0..self.agents.len() {
            while let Ok((len, src)) = self.sockets[i].try_recv_from(&mut buf) {
                if let Ok(Some(reply)) = self.agents[i].handle_incoming_datagram(&buf[..len]) {
                    let _ = self.agents[i].send_lossy_udp(&self.sockets[i], src, &reply, self.udp_loss_rate).await;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
        for i in 0..self.agents.len() {
            while let Ok((len, _)) = self.sockets[i].try_recv_from(&mut buf) { let _ = self.agents[i].handle_incoming_datagram(&buf[..len]); }
            if let Ok(winners) = self.agents[i].finalize_snowball_pulse() {
                if i == 0 && !winners.is_empty() {
                    let w = winners[0];
                    self.push_event(format!("[SNOWBALL] Tick #{}: Đồng thuận chốt giao dịch {:02x}{:02x}..{:02x}{:02x} bất chấp rớt gói UDP!", self.tick, w[0], w[1], w[30], w[31]));
                }
            }
        }
    }
}

fn seed_genesis_allocations(alice: &HumanWallet, bob: &HumanWallet) -> Result<GenesisClusterSeed, DashboardError> {
    let allowance = NanoSkn::from_skn(10)?;
    let (mut keys, mut certs, mut owners, mut outs, mut circ) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), NanoSkn::ZERO);
    for (_, _, skn_whole, is_alice, sub_idx) in &CLUSTER_SPECS {
        let owner = if *is_alice { alice } else { bob };
        owners.push(if *is_alice { "Alice" } else { "Bob" });
        let (hotkey, cert) = owner.authorize_agent(*sub_idx, allowance, 100_000);
        let amt = NanoSkn::from_skn(*skn_whole)?;
        circ = circ.checked_add(amt)?;
        outs.push(TxOutput::agent_bond(owner.human_pubkey(), cert.agent_pubkey, amt, allowance));
        keys.push(hotkey); certs.push(cert);
    }
    let cold_each = NanoSkn::from_skn(100_000)?;
    for owner in [alice, bob] { circ = circ.checked_add(cold_each)?; outs.push(TxOutput::direct(owner.human_pubkey(), cold_each)); }
    Ok((keys, certs, owners, outs, circ))
}

async fn spawn_cluster_agents(specs: &[NodeSpec], keys: &[SigningKey], certs: &[AgentOwnershipCertificate], base_ledger: &UtxoDagLedger) -> Result<(Vec<AgentNode>, Vec<UdpSocket>, Vec<SocketAddr>), DashboardError> {
    let sb = SnowballParams { sample_k: 5, quorum_alpha: 2, finality_beta: 4 };
    let (mut agents, mut sockets, mut addrs) = (Vec::new(), Vec::new(), Vec::new());
    for (i, (name, tier, _, _, _)) in specs.iter().enumerate() {
        let sock = UdpSocket::bind("127.0.0.1:0").await?;
        addrs.push(sock.local_addr()?); sockets.push(sock);
        let mut ag = AgentNode::from_signing_key(*name, 2000 + i as u64, keys[i].clone(), *tier, base_ledger.clone(), sb)?;
        ag.ownership_cert = Some(certs[i].clone());
        agents.push(ag);
    }
    let contacts: Vec<PeerContact> = agents.iter().enumerate().map(|(j, a)| PeerContact {
        pubkey: a.pubkey, addr: addrs[j], cluster_prefix: a.cluster_prefix, prefix_len: a.prefix_len,
        bonded_skn_whole: (a.ledger.balance_of(&a.pubkey).0 / NANO_PER_SKN) as u64, synaptic_score: 100,
    }).collect();
    for (i, ag) in agents.iter_mut().enumerate() {
        for (j, c) in contacts.iter().enumerate() { if i != j { ag.observe_peer_contact(c.clone()); } }
    }
    Ok((agents, sockets, addrs))
}
