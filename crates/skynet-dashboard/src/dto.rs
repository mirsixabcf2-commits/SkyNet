//! `dto`: JSON Data Transfer Objects and pure mapping helpers for `skynet-dashboard`,
//! including the canonical `WorkerStudioTelemetryDto` (`worker_studio_state.json` schema).

use serde::{Deserialize, Serialize};
use skynet_agent::AgentNode;
use skynet_dna::CellRole;
use skynet_ledger::{HumanWallet, NanoSkn, UtxoDagLedger};
use skynet_wire::PeerContact;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyDto {
    pub halving_y1_skn: String,
    pub nutrient_pool_nano: u128,
    pub conservation_ok: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EfficiencyDto {
    pub utreexo_ram_bytes: usize,
    pub total_observed_ips: u64,
    pub avg_stored_ips_per_node: usize,
    pub avg_peer_table_ram_bytes: usize,
    pub naive_ip_ram_bytes: u64,
    pub udp_sent: u64,
    pub udp_dropped: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanDto {
    pub name: String,
    pub cold_vault_skn: String,
    pub net_worth_skn: String,
    pub agents_owned: usize,
    pub allowance_cap: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RolesDto {
    pub stem: usize,
    pub epidermal: usize,
    pub vascular: usize,
    pub visceral: usize,
    pub neural: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDto {
    pub id: usize,
    pub name: String,
    pub owner: String,
    pub tier: String,
    pub cluster_prefix_bin: String,
    pub total_cells: usize,
    pub max_cells: usize,
    pub roles: RolesDto,
    pub mitosis: u64,
    pub apoptosis: u64,
    pub max_gen: u64,
    pub blood_skn: String,
    pub voucher_nano_earned: u128,
    pub plasmids_imported: usize,
    pub stored_peers: usize,
    pub peer_ram_bytes: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpulseTrace {
    pub from_idx: usize,
    pub to_idx: usize,
    pub kind: String,
    pub dropped: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveNodeStatusDto {
    pub is_active: bool,
    pub name: String,
    pub pubkey_hex: String,
    pub tier: String,
    pub epoch: u64,
    pub pulse_seq: u32,
    pub on_chain_balance_skn: String,
    pub blood_pool_skn: String,
    pub voucher_earned_nano: u128,
    pub total_cells: usize,
    pub roles: RolesDto,
    pub stored_peers: usize,
    pub data_dir: String,
}

/// Canonical Worker Earnings & Opportunistic Studio telemetry (`worker_studio_state.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkerStudioTelemetryDto {
    pub governor_state: String,
    pub preempt_latency_ms: f64,
    pub gpu_util_pct: u8,
    pub cpu_util_pct: u8,
    pub vram_used_pct: u8,
    pub temp_c: u8,
    pub on_battery: bool,
    pub active_game_detected: bool,
    pub fountain_shards_completed: u64,
    pub pouw_score_bps: u16,
    pub slashed_count: u32,
    pub earned_nano_skn: u128,
    pub swept_to_cold_nano_skn: u128,
    pub earning_rate_skn_per_hr: f64,
    pub updated_at_sec: u64,
}

impl Default for WorkerStudioTelemetryDto {
    fn default() -> Self {
        Self {
            governor_state: "DeepIdle".into(), preempt_latency_ms: 4.2, gpu_util_pct: 14, cpu_util_pct: 9,
            vram_used_pct: 24, temp_c: 48, on_battery: false, active_game_detected: false,
            fountain_shards_completed: 1, pouw_score_bps: 5_000, slashed_count: 0,
            earned_nano_skn: 0, swept_to_cold_nano_skn: 0, earning_rate_skn_per_hr: 14.5, updated_at_sec: 0,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub tick: u64,
    pub udp_loss_rate: f64,
    pub supply: SupplyDto,
    pub efficiency: EfficiencyDto,
    pub humans: Vec<HumanDto>,
    pub agents: Vec<AgentDto>,
    pub recent_impulses: Vec<ImpulseTrace>,
    pub events: Vec<String>,
    pub live_node: Option<LiveNodeStatusDto>,
    #[serde(default)]
    pub worker_studio: WorkerStudioTelemetryDto,
    #[serde(default)]
    pub yield_sweep_log: Vec<String>,
}

pub fn format_with_commas(mut n: u128) -> String {
    if n == 0 { return "0".to_string(); }
    let mut parts = Vec::new();
    while n > 0 { parts.push(format!("{:03}", n % 1000)); n /= 1000; }
    if let Some(last) = parts.last_mut() { *last = last.trim_start_matches('0').to_string(); }
    parts.reverse();
    parts.join(",")
}

pub fn format_skn_short(v: NanoSkn) -> String {
    format!("{}.{:03} SKN", format_with_commas(v.whole_skn()), (v.sub_nano() / 1_000_000) as u32)
}

pub fn build_efficiency_dto(agents: &[AgentNode], observed_ips: u64) -> EfficiencyDto {
    let cnt = agents.len().max(1);
    EfficiencyDto {
        utreexo_ram_bytes: agents[0].utreexo_roots.ram_footprint_bytes(),
        total_observed_ips: observed_ips,
        avg_stored_ips_per_node: agents.iter().map(|a| a.peer_table.stored_peer_count()).sum::<usize>() / cnt,
        avg_peer_table_ram_bytes: agents.iter().map(|a| a.peer_table.estimated_ram_bytes()).sum::<usize>() / cnt,
        naive_ip_ram_bytes: observed_ips * (std::mem::size_of::<PeerContact>() as u64),
        udp_sent: agents.iter().map(|a| a.udp_sent).sum(),
        udp_dropped: agents.iter().map(|a| a.udp_dropped_simulated).sum(),
    }
}

pub fn build_humans_dto(ledger: &UtxoDagLedger, alice: &HumanWallet, bob: &HumanWallet) -> Vec<HumanDto> {
    vec![
        HumanDto { name: "Alice (Human Cold Key)".into(), cold_vault_skn: format_skn_short(ledger.balance_of(&alice.human_pubkey())), net_worth_skn: format_skn_short(ledger.human_total_net_worth(&alice.human_pubkey())), agents_owned: 4, allowance_cap: "10 SKN / Agent".into() },
        HumanDto { name: "Bob (Human Cold Key)".into(), cold_vault_skn: format_skn_short(ledger.balance_of(&bob.human_pubkey())), net_worth_skn: format_skn_short(ledger.human_total_net_worth(&bob.human_pubkey())), agents_owned: 4, allowance_cap: "10 SKN / Agent".into() },
    ]
}

pub fn build_agents_dto(agents: &[AgentNode], owners: &[&str]) -> Vec<AgentDto> {
    agents.iter().enumerate().map(|(idx, ag)| {
        let st = ag.stats();
        let prefix = if ag.prefix_len == 0 { "0b*".into() } else { format!("0b{:0w$b}", ag.cluster_prefix, w = ag.prefix_len as usize) };
        AgentDto {
            id: idx, name: st.name, owner: owners[idx].into(), tier: format!("{:?}", st.tier), cluster_prefix_bin: prefix,
            total_cells: st.total_cells, max_cells: ag.profile.max_cells,
            roles: RolesDto {
                stem: st.role_counts.get(&CellRole::Stem).copied().unwrap_or(0),
                epidermal: st.role_counts.get(&CellRole::Epidermal).copied().unwrap_or(0),
                vascular: st.role_counts.get(&CellRole::Vascular).copied().unwrap_or(0),
                visceral: st.role_counts.get(&CellRole::Visceral).copied().unwrap_or(0),
                neural: st.role_counts.get(&CellRole::Neural).copied().unwrap_or(0),
            },
            mitosis: st.total_mitosis_events, apoptosis: st.total_apoptosis_events, max_gen: st.max_generation,
            blood_skn: format_skn_short(st.blood_pool), voucher_nano_earned: ag.total_voucher_nano_earned.0,
            plasmids_imported: ag.imported_plasmids, stored_peers: ag.peer_table.stored_peer_count(),
            peer_ram_bytes: ag.peer_table.estimated_ram_bytes(),
        }
    }).collect()
}
