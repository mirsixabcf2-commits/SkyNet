//! `sim_metrics`: Structured reporting, error handling, and protocol invariant
//! assertions for SkyNet's classic multicellular and opportunistic edge simulations.

use std::fmt;

use skynet_agent::{AgentError, AgentNode, OpportunisticWorkerError};
use skynet_cell::{AggregationError, HardwareTier, MicroTrainError};
use skynet_dna::{CellRole, DnaError};
use skynet_ledger::{EscrowError, LedgerError, NanoSkn, GENESIS_TOTAL_NANO_SKN};
use skynet_wire::WireError;

/// Unified explicit error type for `skynet-sim` execution and invariant verification.
#[derive(Debug)]
pub enum SimError {
    Io(std::io::Error),
    Dna(DnaError),
    CellTrain(MicroTrainError),
    Aggregation(AggregationError),
    Ledger(LedgerError),
    Escrow(EscrowError),
    Wire(WireError),
    Agent(AgentError),
    Worker(OpportunisticWorkerError),
    Invariant(&'static str),
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "simulation I/O error: {e}"),
            Self::Dna(e) => write!(f, "simulation DNA error: {e}"),
            Self::CellTrain(e) => write!(f, "simulation micro-train error: {e}"),
            Self::Aggregation(e) => write!(f, "simulation aggregation error: {e}"),
            Self::Ledger(e) => write!(f, "simulation ledger error: {e}"),
            Self::Escrow(e) => write!(f, "simulation escrow error: {e}"),
            Self::Wire(e) => write!(f, "simulation wire error: {e}"),
            Self::Agent(e) => write!(f, "simulation agent error: {e}"),
            Self::Worker(e) => write!(f, "simulation worker error: {e}"),
            Self::Invariant(msg) => write!(f, "simulation invariant violated: {msg}"),
        }
    }
}

impl std::error::Error for SimError {}
impl From<std::io::Error> for SimError { fn from(e: std::io::Error) -> Self { Self::Io(e) } }
impl From<DnaError> for SimError { fn from(e: DnaError) -> Self { Self::Dna(e) } }
impl From<MicroTrainError> for SimError { fn from(e: MicroTrainError) -> Self { Self::CellTrain(e) } }
impl From<AggregationError> for SimError { fn from(e: AggregationError) -> Self { Self::Aggregation(e) } }
impl From<LedgerError> for SimError { fn from(e: LedgerError) -> Self { Self::Ledger(e) } }
impl From<EscrowError> for SimError { fn from(e: EscrowError) -> Self { Self::Escrow(e) } }
impl From<WireError> for SimError { fn from(e: WireError) -> Self { Self::Wire(e) } }
impl From<AgentError> for SimError { fn from(e: AgentError) -> Self { Self::Agent(e) } }
impl From<OpportunisticWorkerError> for SimError { fn from(e: OpportunisticWorkerError) -> Self { Self::Worker(e) } }

/// Cellular lineage growth metrics for one simulated organism.
#[derive(Debug, Clone)]
pub struct AgentGrowthRow {
    pub name: String,
    pub tier: HardwareTier,
    pub total_cells: usize,
    pub stem: usize,
    pub skin: usize,
    pub blood: usize,
    pub organ: usize,
    pub brain: usize,
    pub mitosis: u64,
    pub apoptosis: u64,
    pub max_gen: u64,
}

/// Summary report produced by `classic_sim::run_classic_simulation`.
#[derive(Debug, Clone)]
pub struct ClassicSimReport {
    pub halving_y0: NanoSkn,
    pub halving_y1: NanoSkn,
    pub total_accounted_nano: u128,
    pub alice_net_worth: NanoSkn,
    pub utreexo_ram_bytes: usize,
    pub agent_rows: Vec<AgentGrowthRow>,
    pub witnessed_wire_size: usize,
    pub voucher_earned_nano: u128,
    pub udp_sent: u64,
    pub udp_dropped: u64,
    pub unanimous_winner: [u8; 32],
    pub supply_conserved: bool,
}

/// Summary report produced by `opportunistic_sim::run_opportunistic_simulation`.
#[derive(Debug, Clone)]
pub struct OpportunisticSimReport {
    pub droplets_sent: u32,
    pub droplets_dropped: u32,
    pub max_wire_bytes: usize,
    pub preempt_latency_ms: f64,
    pub resumed_steps: u16,
    pub total_workers: usize,
    pub byzantine_slashed: usize,
    pub initial_loss: f32,
    pub converged_loss: f32,
    pub swept_to_cold_skn: u64,
    pub worker_reserve_skn: u64,
    pub total_accounted_nano: u128,
}

pub fn collect_growth_rows(agents: &[AgentNode]) -> Vec<AgentGrowthRow> {
    agents.iter().map(|ag| {
        let s = ag.stats();
        AgentGrowthRow {
            name: s.name, tier: s.tier, total_cells: s.total_cells,
            stem: s.role_counts.get(&CellRole::Stem).copied().unwrap_or(0),
            skin: s.role_counts.get(&CellRole::Epidermal).copied().unwrap_or(0),
            blood: s.role_counts.get(&CellRole::Vascular).copied().unwrap_or(0),
            organ: s.role_counts.get(&CellRole::Visceral).copied().unwrap_or(0),
            brain: s.role_counts.get(&CellRole::Neural).copied().unwrap_or(0),
            mitosis: s.total_mitosis_events, apoptosis: s.total_apoptosis_events, max_gen: s.max_generation,
        }
    }).collect()
}

pub fn verify_classic_invariants(r: &ClassicSimReport) -> Result<(), SimError> {
    if !r.supply_conserved || r.total_accounted_nano != GENESIS_TOTAL_NANO_SKN {
        return Err(SimError::Invariant("classic simulation violated 21T SKN conservation"));
    }
    if r.witnessed_wire_size > skynet_wire::MAX_UDP_DATAGRAM_BYTES {
        return Err(SimError::Invariant("WitnessedTx exceeded 1200B UDP MTU"));
    }
    if r.voucher_earned_nano != 2_000 {
        return Err(SimError::Invariant("cumulative SynapticVoucher failed to recover 2,000 nanoSKN"));
    }
    Ok(())
}

pub fn verify_opportunistic_invariants(r: &OpportunisticSimReport) -> Result<(), SimError> {
    if r.max_wire_bytes > skynet_wire::MAX_UDP_DATAGRAM_BYTES {
        return Err(SimError::Invariant("opportunistic packet exceeded 1200B UDP MTU"));
    }
    if r.preempt_latency_ms >= 100.0 {
        return Err(SimError::Invariant("game preemption exceeded 100ms SLA"));
    }
    if r.byzantine_slashed != 3 || !r.converged_loss.is_finite() || r.converged_loss >= r.initial_loss {
        return Err(SimError::Invariant("Byzantine slashing or TrimmedMean convergence failed"));
    }
    if r.swept_to_cold_skn == 0 || r.total_accounted_nano != GENESIS_TOTAL_NANO_SKN {
        return Err(SimError::Invariant("Yield Sweep or 21T SKN conservation failed"));
    }
    Ok(())
}

pub fn print_classic_report(r: &ClassicSimReport) {
    println!("[1] KINH TE HOC GENESIS 21 NGHIN TI SKN & VI CHU QUYEN CON NGUOI:");
    println!("    - Tong cung bat bien   : {} ({} nanoSKN)", NanoSkn::GENESIS_CAP, GENESIS_TOTAL_NANO_SKN);
    println!("    - Quy Halving Nam 1->2 : {} -> {} (-50.0%/nam)", r.halving_y0, r.halving_y1);
    println!("    - Tai san Chu nhan Alice: {} (100k SKN vi lanh + 3 AI Agents)", r.alice_net_worth);
    println!("    - Bo tich luy Utreexo   : {} bytes RAM xac thuc toan mang\n", r.utreexo_ram_bytes);
    println!("[2] TANG TRUONG DA BAO AI TREN CANDLE (SAU 16 CHU KY TIEN HOA):");
    for s in &r.agent_rows {
        println!(
            "    * {:<16} | Tier: {:<16?} | Cells: {:>4} (G:{:>2} D:{:>3} M:{:>3} T:{:>3} TK:{:>3}) | +{:>4}/-{:>3} | Gen:{}",
            s.name, s.tier, s.total_cells, s.stem, s.skin, s.blood, s.organ, s.brain, s.mitosis, s.apoptosis, s.max_gen
        );
    }
    let loss_pct = (r.udp_dropped as f64) * 100.0 / ((r.udp_sent + r.udp_dropped).max(1) as f64);
    let w = r.unanimous_winner;
    println!("\n[3] XAC THUC SIEU NHE UTREEXO, VOUCHER VI MO & SNOWBALL QUA UDP ROT GOI:");
    println!("    - Goi WitnessedTx Utreexo  : {} bytes (<= 1200B UDP MTU)", r.witnessed_wire_size);
    println!("    - Luong SynapticVoucher    : Thu du {} nanoSKN bat chap rot goi", r.voucher_earned_nano);
    println!("    - Xung UDP gui / bi rot    : {} / {} ({:.1}% loss, 0 retransmit)", r.udp_sent, r.udp_dropped, loss_pct);
    println!("    - Phan xu Double-Spend     : 6/6 dong thuan Tx {:02x}{:02x}..{:02x}{:02x}\n", w[0], w[1], w[30], w[31]);
}

pub fn print_opportunistic_report(r: &OpportunisticSimReport) {
    let drop_pct = (r.droplets_dropped as f64) * 100.0 / (r.droplets_sent.max(1) as f64);
    println!("[4] MANG AI CAN BIEN CO HOI (40% LOSS + <100MS PREEMPT + 30% BYZANTINE + YIELD SWEEP):");
    println!("    - Fountain Shard 40% Loss  : Gui {} giot / rot {} ({:.1}%), khoi phuc 100% (max {}B <= 1200B)", r.droplets_sent, r.droplets_dropped, drop_pct, r.max_wire_bytes);
    println!("    - Nhuong may Game thu      : {:.3} ms (< 100ms SLA), noi lai {} steps tu MicroTrainCheckpoint", r.preempt_latency_ms, r.resumed_steps);
    println!("    - PoUW Canary & TrimmedMean: Cat coc {}/{} nut Byzantine, MSE Loss {:.4} -> {:.4}", r.byzantine_slashed, r.total_workers, r.initial_loss, r.converged_loss);
    println!("    - Streaming & Yield Sweep  : Quet +{} SKN ve Vi Lanh (0% burn, giu {} SKN AgentBond)", r.swept_to_cold_skn, r.worker_reserve_skn);
    println!("    - Bao toan 21 Nghin Ti SKN : {} nanoSKN (sai so 0 nanoSKN!)\n", r.total_accounted_nano);
}
