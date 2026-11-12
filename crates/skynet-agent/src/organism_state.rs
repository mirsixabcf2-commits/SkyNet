//! `organism_state`: Multicellular `AgentNode` state, homeostasis, mitosis/apoptosis, and `NanoSkn` invariants.

use std::collections::{HashMap, HashSet};
use candle_core::Device;
use ed25519_dalek::SigningKey;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use skynet_cell::{HardwareProfile, HardwareTier, MicroCell, TissueBatch};
use skynet_dna::{CellDna, CellRole, DEFAULT_INPUT_DIM};
use skynet_ledger::{AgentMetabolicBank, AgentOwnershipCertificate, NanoSkn, StatelessUtreexoRoots, UtxoDagLedger, UtxoId, VoucherChannelTracker};
use skynet_wire::{FountainDecoder, PeerContact, SynapticPeerTable};

use crate::compute_flow::ComputeFlowCoordinator;
use crate::opportunistic_worker::OpportunisticWorkerEngine;
use crate::rendezvous_broker::RendezvousRegistry;
use crate::snowball::{SnowballEngine, SnowballParams};
pub use crate::{AgentError, OrganismStats, PendingSnowballQuery, RestoreOrganismOptions};

/// 1 Node = 1 Multicellular AI `AgentNode` living on a CPU/GPU device and speaking lossy UDP.
pub struct AgentNode {
    pub name: String,
    pub signing_key: SigningKey,
    pub pubkey: [u8; 32],
    pub ownership_cert: Option<AgentOwnershipCertificate>,
    pub profile: HardwareProfile,
    pub device: Device,
    pub cluster_prefix: u32,
    pub prefix_len: u8,
    pub epoch: u64,
    pub pulse_seq: u32,
    pub cells: Vec<MicroCell>,
    pub bank: AgentMetabolicBank,
    pub ledger: UtxoDagLedger,
    pub utreexo_roots: StatelessUtreexoRoots,
    pub voucher_channels: HashMap<UtxoId, VoucherChannelTracker>,
    pub total_voucher_nano_earned: NanoSkn,
    pub peer_table: SynapticPeerTable,
    pub snowball: SnowballEngine,
    pub compute_coordinator: ComputeFlowCoordinator,
    pub rendezvous: RendezvousRegistry,
    pub opportunistic_worker: OpportunisticWorkerEngine,
    pub(crate) next_cell_id: u64,
    pub(crate) rng: ChaCha8Rng,
    pub total_mitosis_events: u64,
    pub total_apoptosis_events: u64,
    pub udp_sent: u64,
    pub udp_received: u64,
    pub udp_dropped_simulated: u64,
    pub(crate) fountain_decoders: HashMap<[u8; 32], FountainDecoder>,
    pub imported_plasmids: usize,
    pub pending_snowball_queries: HashMap<u64, PendingSnowballQuery>,
    pub cluster_mitosis_votes: HashMap<(u32, u8), HashSet<[u8; 32]>>,
}

impl AgentNode {
    pub fn spawn_organism(name: impl Into<String>, seed: u64, tier: HardwareTier, ledger: UtxoDagLedger, snowball_params: SnowballParams) -> Result<Self, AgentError> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        Self::from_signing_key(name, seed, SigningKey::generate(&mut rng), tier, ledger, snowball_params)
    }

    pub fn from_signing_key(name: impl Into<String>, seed: u64, signing_key: SigningKey, tier: HardwareTier, ledger: UtxoDagLedger, snowball_params: SnowballParams) -> Result<Self, AgentError> {
        let pubkey = signing_key.verifying_key().to_bytes();
        let profile = HardwareProfile::for_tier(tier);
        let on_chain = ledger.balance_of(&pubkey);
        let mut bank = AgentMetabolicBank::new(on_chain);
        let cells = seed_initial_cells(seed, &profile, on_chain, &mut bank)?;
        Self::restore_organism(RestoreOrganismOptions { name: name.into(), seed, signing_key, tier, ledger, cells, bank, snowball_params })
    }

    pub fn restore_organism(opts: RestoreOrganismOptions) -> Result<Self, AgentError> {
        let pubkey = opts.signing_key.verifying_key().to_bytes();
        let (max_intra, max_k) = peer_limits_for_tier(opts.tier);
        let max_id = opts.cells.iter().map(|c| c.cell_id).max().unwrap_or(0);
        let node = Self {
            name: opts.name, signing_key: opts.signing_key, pubkey, ownership_cert: None,
            profile: HardwareProfile::for_tier(opts.tier), device: HardwareProfile::select_compute_device(),
            cluster_prefix: 0, prefix_len: 0, epoch: 0, pulse_seq: 0, utreexo_roots: opts.ledger.build_utreexo_forest().roots,
            cells: opts.cells, bank: opts.bank, ledger: opts.ledger, voucher_channels: HashMap::new(),
            total_voucher_nano_earned: NanoSkn::ZERO, peer_table: SynapticPeerTable::new(&pubkey, 0, 0, max_intra, max_k),
            snowball: SnowballEngine::new(opts.snowball_params), compute_coordinator: ComputeFlowCoordinator::new(),
            rendezvous: RendezvousRegistry::new(), opportunistic_worker: OpportunisticWorkerEngine::default(),
            next_cell_id: max_id + 1, rng: ChaCha8Rng::seed_from_u64(opts.seed ^ 0xA5A5_5A5A),
            total_mitosis_events: 0, total_apoptosis_events: 0, udp_sent: 0, udp_received: 0, udp_dropped_simulated: 0,
            fountain_decoders: HashMap::new(), imported_plasmids: 0, pending_snowball_queries: HashMap::new(), cluster_mitosis_votes: HashMap::new(),
        };
        node.verify_internal_skn_conservation()?;
        Ok(node)
    }

    pub fn sync_blood_with_ledger(&mut self) -> Result<(), AgentError> {
        let on_chain = self.ledger.balance_of(&self.pubkey);
        let cell_sum = self.cells.iter().try_fold(NanoSkn::ZERO, |a, c| a.checked_add(c.energy))?;
        if on_chain >= cell_sum {
            self.bank.blood_pool = on_chain.checked_sub(cell_sum)?;
            return self.verify_internal_skn_conservation();
        }
        let mut deficit = cell_sum.checked_sub(on_chain)?.0;
        self.bank.blood_pool = NanoSkn::ZERO;
        for c in self.cells.iter_mut().rev() {
            let take = deficit.min(c.energy.0);
            c.energy = NanoSkn(c.energy.0 - take);
            deficit -= take;
        }
        self.verify_internal_skn_conservation()
    }

    pub fn verify_internal_skn_conservation(&self) -> Result<(), AgentError> {
        self.bank.verify_invariant(self.ledger.balance_of(&self.pubkey), self.cells.iter().map(|c| c.energy))?;
        Ok(())
    }

    pub fn run_biological_tick(&mut self, stimulus: &[f32; DEFAULT_INPUT_DIM], target_signal: &[f32; 4]) -> Result<Vec<f32>, AgentError> {
        self.pulse_seq = self.pulse_seq.wrapping_add(1);
        for cell in &mut self.cells {
            let burned = cell.metabolize_tick(self.profile.basal_burn_per_tick);
            self.bank.deposit_to_blood(burned)?;
        }
        let batch_inputs: Vec<Vec<f32>> = (0..self.cells.len()).map(|_| stimulus.to_vec()).collect();
        let outputs = TissueBatch::forward_slice(&mut self.cells, &batch_inputs, &self.device)?;
        let cortical = self.score_and_feed_cells(&outputs, target_signal)?;
        self.differentiate_and_divide()?;
        self.prune_apoptotic_cells()?;
        self.verify_internal_skn_conservation()?;
        Ok(cortical)
    }

    fn score_and_feed_cells(&mut self, outputs: &[Vec<f32>], target: &[f32; 4]) -> Result<Vec<f32>, AgentError> {
        let (mut quals, mut role_q, mut cort, mut n_cort) = (Vec::with_capacity(outputs.len()), HashMap::new(), vec![0.0f32; 4], 0usize);
        for (cell, out) in self.cells.iter().zip(outputs.iter()) {
            if cell.role() == CellRole::Neural {
                for (dst, &v) in cort.iter_mut().zip(out.iter().take(4)) { *dst += v; }
                n_cort += 1;
            }
            let q = (1.0 - out.iter().zip(target.iter()).map(|(y, t)| (y - t).abs()).sum::<f32>() / 4.0).clamp(0.0, 1.0);
            let e = role_q.entry(cell.role()).or_insert((0.0f32, 0usize));
            *e = (e.0 + q, e.1 + 1);
            quals.push(q);
        }
        for (cell, &q) in self.cells.iter_mut().zip(quals.iter()) {
            let (sq, cq) = role_q.get(&cell.role()).copied().unwrap_or((0.0, 1));
            if cell.role() == CellRole::Stem || cq <= 2 || q >= (sq / cq.max(1) as f32) * 0.98 {
                let avail = (((q * 320.0) as u128) + self.profile.basal_burn_per_tick.0).min(self.bank.blood_pool.0 / 8);
                if avail > 0 { cell.absorb_energy(self.bank.draw_from_blood(NanoSkn(avail))?, q)?; }
            } else {
                cell.health = (cell.health - 0.35).max(0.0);
            }
        }
        if n_cort > 0 { for v in &mut cort { *v /= n_cort as f32; } }
        Ok(cort)
    }

    fn differentiate_and_divide(&mut self) -> Result<(), AgentError> {
        let needed = self.find_most_needed_role();
        if let (Some(role), true) = (needed, self.cells.iter().filter(|c| c.role() == CellRole::Stem).count() > 1) {
            if let Some(stem) = self.cells.iter_mut().find(|c| c.role() == CellRole::Stem) { stem.differentiate(role); }
        }
        let (mut newborns, mut slots) = (Vec::new(), self.profile.max_cells.saturating_sub(self.cells.len()));
        for cell in &mut self.cells {
            if slots == 0 || !cell.can_divide(&self.profile) { continue; }
            let (mut daughter, fee) = cell.divide_mitosis(self.next_cell_id, &self.profile, &mut self.rng)?;
            self.next_cell_id += 1;
            self.bank.deposit_to_blood(fee)?;
            if daughter.role() == CellRole::Stem { if let Some(r) = needed { daughter.differentiate(r); } }
            newborns.push(daughter);
            self.total_mitosis_events += 1;
            slots -= 1;
        }
        self.cells.extend(newborns);
        Ok(())
    }

    fn prune_apoptotic_cells(&mut self) -> Result<(), AgentError> {
        let (mut surviving, mut kept) = (Vec::with_capacity(self.cells.len()), HashMap::new());
        for cell in self.cells.drain(..) {
            let role = cell.role();
            if kept.get(&role).copied().unwrap_or(0usize) == 0 || !cell.should_apoptose(&self.profile) {
                *kept.entry(role).or_default() += 1;
                surviving.push(cell);
                continue;
            }
            self.bank.deposit_to_blood(cell.energy)?;
            self.total_apoptosis_events += 1;
        }
        self.cells = surviving;
        Ok(())
    }

    fn find_most_needed_role(&self) -> Option<CellRole> {
        let mut counts: HashMap<CellRole, usize> = HashMap::new();
        for c in &self.cells { *counts.entry(c.role()).or_default() += 1; }
        CellRole::ALL_SPECIALIZED.into_iter().min_by_key(|r| counts.get(r).copied().unwrap_or(0))
    }

    pub fn split_cluster_prefix(&mut self) {
        if self.prefix_len >= 30 { return; }
        let idx = self.prefix_len as usize;
        self.cluster_prefix = (self.cluster_prefix << 1) | (((self.pubkey[idx / 8] >> (7 - (idx % 8))) & 1) as u32);
        self.prefix_len += 1;
        self.peer_table.local_prefix = self.cluster_prefix;
        self.peer_table.local_prefix_len = self.prefix_len;
    }

    pub fn observe_peer_contact(&mut self, contact: PeerContact) { self.peer_table.observe_peer(contact); }
}

fn peer_limits_for_tier(tier: HardwareTier) -> (usize, usize) {
    match tier { HardwareTier::TinyPeripheral => (10, 2), HardwareTier::StandardNode => (24, 3), HardwareTier::ColossalOrganism => (48, 4) }
}

fn seed_initial_cells(seed: u64, profile: &HardwareProfile, on_chain: NanoSkn, bank: &mut AgentMetabolicBank) -> Result<Vec<MicroCell>, AgentError> {
    let roles = [CellRole::Stem, CellRole::Stem, CellRole::Epidermal, CellRole::Vascular, CellRole::Visceral, CellRole::Neural];
    let energy = NanoSkn::from_nano((profile.mitosis_threshold.0 * 6 / 5).min(on_chain.0 / 20));
    let mut cells = Vec::new();
    for (i, &role) in roles.iter().enumerate() {
        if bank.blood_pool < energy || energy.0 == 0 { continue; }
        let stem = CellDna::genesis_stem(seed.wrapping_add(i as u64 * 31));
        let dna = if role == CellRole::Stem { stem } else { stem.differentiate(role) };
        cells.push(MicroCell::spawn((i + 1) as u64, dna, bank.draw_from_blood(energy)?)?);
    }
    if let Some(first) = cells.first_mut() { first.dna = CellDna::genesis_stem(seed); }
    Ok(cells)
}
