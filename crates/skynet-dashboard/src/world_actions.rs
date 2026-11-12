//! `world_actions`: Interactive chaos protocol controls (`SetLoss`, `FloodIps`,
//! `DoubleSpend`, `ClusterMitosis`, `ReclaimIot`) and local live-node storage reader.

use std::net::{Ipv4Addr, SocketAddr};

use rand::Rng;
use skynet_dna::CellRole;
use skynet_ledger::{NanoSkn, SknTransaction, TxOutput};
use skynet_node::StorageEngine;
use skynet_wire::PeerContact;

use crate::dto::{format_skn_short, LiveNodeStatusDto, RolesDto};
use crate::live_world::LiveWorld;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlCmd {
    SetLoss(f64),
    FloodIps,
    DoubleSpend,
    ClusterMitosis,
    ReclaimIot,
}

impl LiveWorld {
    pub fn apply_control(&mut self, cmd: ControlCmd) {
        match cmd {
            ControlCmd::SetLoss(rate) => {
                self.udp_loss_rate = rate.clamp(0.0, 0.60);
                self.push_event(format!(
                    "[CHAOS] Đặt tỷ lệ rớt gói UDP sang {:.0}%",
                    self.udp_loss_rate * 100.0
                ));
            }
            ControlCmd::FloodIps => self.flood_synthetic_ips(),
            ControlCmd::ClusterMitosis => self.trigger_cluster_mitosis(),
            ControlCmd::DoubleSpend => self.inject_double_spend(),
            ControlCmd::ReclaimIot => self.reclaim_iot_bond(),
        }
    }

    fn flood_synthetic_ips(&mut self) {
        for _ in 0..5_000u32 {
            self.synthetic_ip_counter += 1;
            let c_id = self.synthetic_ip_counter;
            let mut pk = [0u8; 32];
            self.rng.fill(&mut pk);
            let ip = Ipv4Addr::new(172, ((c_id >> 16) & 255) as u8, ((c_id >> 8) & 255) as u8, (c_id & 255) as u8);
            let addr = SocketAddr::from((ip, (10_000 + (c_id % 40_000)) as u16));
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
        let avg_ram = self.agents.iter().map(|a| a.peer_table.estimated_ram_bytes()).sum::<usize>() / self.agents.len().max(1);
        self.push_event(format!(
            "[ROUTING] Đã bơm +5.000 IP mới (tổng {} IP)! Bảng Kademlia O(log N) nén chỉ tốn TB {} bytes/node!",
            self.synthetic_ip_counter, avg_ram
        ));
    }

    fn trigger_cluster_mitosis(&mut self) {
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

    fn inject_double_spend(&mut self) {
        let Some((utxo_id, utxo_out)) = self.agents[0].ledger.utxos_for_owner(&self.agents[0].pubkey).first().cloned() else { return };
        let send_amt = NanoSkn::from_nano(500_000_000);
        let fee = NanoSkn::ONE_NANO;
        let Ok(rem) = utxo_out.amount.checked_sub(NanoSkn::from_nano(500_000_001)) else { return };
        let Ok(next_allow) = NanoSkn::from_skn(9) else { return };
        let tips = self.agents[0].ledger.finalized_tips();
        let bond_change = TxOutput::agent_bond(self.human_alice.human_pubkey(), self.agents[0].pubkey, rem, next_allow);

        let tx_a = SknTransaction::new_signed(
            &self.keys[0],
            vec![utxo_id],
            vec![TxOutput::direct(self.human_bob.human_pubkey(), send_amt), bond_change.clone()],
            fee,
            tips.clone(),
            self.tick + 100,
        );
        let tx_b = SknTransaction::new_signed(
            &self.keys[0],
            vec![utxo_id],
            vec![TxOutput::direct(self.agents[7].pubkey, send_amt), bond_change],
            fee,
            tips,
            self.tick + 200,
        );
        for (idx, ag) in self.agents.iter_mut().enumerate() {
            let (first, second) = if idx < 5 { (tx_a.clone(), tx_b.clone()) } else { (tx_b.clone(), tx_a.clone()) };
            if ag.ledger.admit_pending_tx(first.clone()).is_ok() {
                ag.snowball.register_tx(&first);
            }
            let _ = ag.ledger.admit_pending_tx(second);
        }
        self.push_event("[CONFLICT] Đã bơm cặp giao dịch Double-Spend xung đột vào mạng UDP để Snowball phân xử!".into());
    }

    fn reclaim_iot_bond(&mut self) {
        if self.iot_reclaimed { return; }
        let Some((bond_id, bond_out)) = self.agents[5].ledger.utxos_for_owner(&self.agents[5].pubkey).first().cloned() else { return };
        let tips = self.agents[5].ledger.finalized_tips();
        let Ok(reclaim_tx) = self.human_alice.build_sovereign_reclaim_tx(bond_id, bond_out.amount, NanoSkn::ONE_NANO, tips, 9999) else { return };
        let h = reclaim_tx.tx_hash();
        for ag in &mut self.agents {
            if ag.ledger.admit_pending_tx(reclaim_tx.clone()).is_ok() {
                let _ = ag.ledger.finalize_tx(&h);
                let _ = ag.sync_blood_with_ledger();
            }
        }
        self.iot_reclaimed = true;
        self.push_event("[SOVEREIGN] Chủ nhân Alice đã ký Khóa Lạnh thu hồi khẩn cấp 100% vốn từ IoT-Edge-06 về Ví Lạnh!".into());
    }
}

pub fn detect_local_live_node() -> Option<LiveNodeStatusDto> {
    let data_dir = resolve_skynet_data_dir();
    let storage = StorageEngine::new(&data_dir).ok()?;
    if !storage.has_existing_state() { return None; }
    let meta = storage.load_metadata().ok()??;
    let org = storage.load_organism().ok()??;
    let ledger = storage.load_ledger().ok()??;
    let peers = storage.load_peers().unwrap_or_default();

    let (mut stem, mut epidermal, mut vascular, mut visceral, mut neural) = (0, 0, 0, 0, 0);
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
        pubkey_hex: meta.pubkey.iter().map(|b| format!("{b:02x}")).collect(),
        tier: format!("{:?}", meta.tier),
        epoch: meta.epoch,
        pulse_seq: meta.pulse_seq,
        on_chain_balance_skn: format_skn_short(on_chain),
        blood_pool_skn: format_skn_short(org.blood_pool),
        voucher_earned_nano: org.total_voucher_nano_earned.0,
        total_cells: org.cells.len(),
        roles: RolesDto { stem, epidermal, vascular, visceral, neural },
        stored_peers: peers.len(),
        data_dir: data_dir.to_string_lossy().to_string(),
    })
}

pub fn resolve_skynet_data_dir() -> std::path::PathBuf {
    std::env::var("SKYNET_DATA_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home).join(".skynet").join("data")
    })
}
