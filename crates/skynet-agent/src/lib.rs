//! `skynet-agent`: Complete Multicellular AI `AgentNode` combining `candle` micro-cells,
//! homeostatic Stem differentiation, Darwinian Mitosis/Apoptosis, two-tier `NanoSkn` conservation,
//! and loss-tolerant Snowball consensus over connectionless UDP sockets.

use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;

use candle_core::Device;
use ed25519_dalek::SigningKey;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use skynet_cell::{CellError, HardwareProfile, HardwareTier, MicroCell, TissueBatch};
use skynet_dna::{CellDna, CellRole, DEFAULT_INPUT_DIM};
use skynet_ledger::{
    AgentMetabolicBank, AgentOwnershipCertificate, CustodyPolicy, LedgerError, NanoSkn,
    SknTransaction, StatelessUtreexoRoots, UtxoDagLedger, UtxoId, VoucherChannelTracker,
};
use skynet_wire::{
    FountainDecoder, FountainEncoder, ImpulseBody, ImpulsePacket, PeerContact, SynapticPeerTable,
    WireError, DEFAULT_PORT, MAX_UDP_DATAGRAM_BYTES,
};
use thiserror::Error;
use tokio::net::UdpSocket;

pub mod compute_flow;
pub use compute_flow::{ComputeFlowCoordinator, ComputeFlowError};

pub mod auditor_flow;
pub use auditor_flow::AuditorCoordinator;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("cell error: {0}")]
    Cell(#[from] CellError),
    #[error("ledger error: {0}")]
    Ledger(#[from] LedgerError),
    #[error("wire error: {0}")]
    Wire(#[from] WireError),
    #[error("compute flow error: {0}")]
    Compute(#[from] ComputeFlowError),
    #[error("UDP I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Snowball consensus parameters tuned for lossy UDP sub-sampling.
#[derive(Debug, Clone, Copy)]
pub struct SnowballParams {
    /// Number of random peers queried per UDP pulse (`k`).
    pub sample_k: usize,
    /// Minimum agreeing replies required to advance confidence (`alpha >= 2/3` of surviving quorum).
    pub quorum_alpha: usize,
    /// Consecutive quorum rounds required for irreversible finality (`beta`).
    pub finality_beta: u16,
}

impl Default for SnowballParams {
    fn default() -> Self {
        Self {
            sample_k: 5,
            quorum_alpha: 3,
            finality_beta: 5,
        }
    }
}

#[derive(Debug, Clone)]
struct ConflictPreference {
    preferred_tx: [u8; 32],
    last_round_winner: Option<[u8; 32]>,
    confidence: u16,
}

/// Tracks metastable Snowball convergence across contested UTXOs without TCP retransmissions.
/// Retains finalized decisions in `decided` so slower peers suffering UDP packet loss still
/// receive authoritative votes and converge cleanly.
#[derive(Debug, Clone)]
pub struct SnowballEngine {
    pub params: SnowballParams,
    preferences: BTreeMap<UtxoId, ConflictPreference>,
    decided: BTreeMap<UtxoId, [u8; 32]>,
    active_round_votes: BTreeMap<UtxoId, Vec<[u8; 32]>>,
}

impl SnowballEngine {
    pub fn new(params: SnowballParams) -> Self {
        Self {
            params,
            preferences: BTreeMap::new(),
            decided: BTreeMap::new(),
            active_round_votes: BTreeMap::new(),
        }
    }

    pub fn register_tx(&mut self, tx: &SknTransaction) {
        let tx_hash = tx.tx_hash();
        for inp in &tx.inputs {
            if self.decided.contains_key(inp) {
                continue;
            }
            self.preferences
                .entry(*inp)
                .or_insert(ConflictPreference {
                    preferred_tx: tx_hash,
                    last_round_winner: None,
                    confidence: 0,
                });
        }
    }

    pub fn contested_utxos(&self) -> Vec<UtxoId> {
        self.preferences.keys().copied().collect()
    }

    /// Returns either the finalized winner (if already decided) or current metastable preference.
    pub fn vote_for(&self, utxo: &UtxoId) -> Option<[u8; 32]> {
        self.decided
            .get(utxo)
            .copied()
            .or_else(|| self.preferences.get(utxo).map(|p| p.preferred_tx))
    }

    pub fn decided_for(&self, utxo: &UtxoId) -> Option<[u8; 32]> {
        self.decided.get(utxo).copied()
    }

    pub fn record_incoming_vote(&mut self, utxo: UtxoId, voted_tx: [u8; 32]) {
        if self.preferences.contains_key(&utxo) {
            self.active_round_votes
                .entry(utxo)
                .or_default()
                .push(voted_tx);
        }
    }

    /// Closes the current UDP pulse window and returns newly finalized transaction hashes.
    pub fn close_pulse_window(&mut self) -> Vec<[u8; 32]> {
        let collected = std::mem::take(&mut self.active_round_votes);
        let mut newly_finalized = Vec::new();
        for (utxo, votes) in collected {
            if let Some(winner) = self.observe_sample_tally(utxo, &votes) {
                if !newly_finalized.contains(&winner) {
                    newly_finalized.push(winner);
                }
            }
        }
        newly_finalized
    }

    /// Evaluates surviving UDP sample replies for `utxo` (ignoring dropped UDP packets!).
    pub fn observe_sample_tally(
        &mut self,
        utxo: UtxoId,
        votes: &[[u8; 32]],
    ) -> Option<[u8; 32]> {
        let pref = self.preferences.get_mut(&utxo)?;
        let mut counts: HashMap<[u8; 32], usize> = HashMap::new();
        for v in votes {
            *counts.entry(*v).or_default() += 1;
        }

        let best = counts
            .into_iter()
            .max_by_key(|(_, cnt)| *cnt)
            .filter(|(_, cnt)| *cnt >= self.params.quorum_alpha);

        if let Some((winner, _)) = best {
            pref.preferred_tx = winner;
            if pref.last_round_winner == Some(winner) {
                pref.confidence = pref.confidence.saturating_add(1);
            } else {
                pref.last_round_winner = Some(winner);
                pref.confidence = 1;
            }
            if pref.confidence >= self.params.finality_beta {
                self.preferences.remove(&utxo);
                self.decided.insert(utxo, winner);
                return Some(winner);
            }
        }
        None
    }
}

/// Summary snapshot of an Agent's multicellular body.
#[derive(Debug, Clone)]
pub struct OrganismStats {
    pub name: String,
    pub tier: HardwareTier,
    pub cluster_prefix: u32,
    pub prefix_len: u8,
    pub total_cells: usize,
    pub role_counts: HashMap<CellRole, usize>,
    pub max_generation: u64,
    pub on_chain_balance: NanoSkn,
    pub blood_pool: NanoSkn,
    pub total_mitosis_events: u64,
    pub total_apoptosis_events: u64,
    pub udp_sent: u64,
    pub udp_received: u64,
    pub udp_dropped_simulated: u64,
}

pub type PendingSnowballQuery = (u64, std::collections::HashSet<[u8; 32]>, Vec<UtxoId>);

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

    next_cell_id: u64,
    rng: ChaCha8Rng,
    pub total_mitosis_events: u64,
    pub total_apoptosis_events: u64,
    pub udp_sent: u64,
    pub udp_received: u64,
    pub udp_dropped_simulated: u64,
    fountain_decoders: HashMap<[u8; 32], FountainDecoder>,
    pub imported_plasmids: usize,
    pub pending_snowball_queries: HashMap<u64, PendingSnowballQuery>,
    pub cluster_mitosis_votes: HashMap<(u32, u8), std::collections::HashSet<[u8; 32]>>,
}

/// Parameters for restoring a persisted multicellular organism.
#[derive(Debug)]
pub struct RestoreOrganismOptions {
    pub name: String,
    pub seed: u64,
    pub signing_key: SigningKey,
    pub tier: HardwareTier,
    pub ledger: UtxoDagLedger,
    pub cells: Vec<MicroCell>,
    pub bank: AgentMetabolicBank,
    pub snowball_params: SnowballParams,
}

impl AgentNode {
    pub fn spawn_organism(
        name: impl Into<String>,
        seed: u64,
        tier: HardwareTier,
        ledger: UtxoDagLedger,
        snowball_params: SnowballParams,
    ) -> Result<Self, AgentError> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let signing_key = SigningKey::generate(&mut rng);
        Self::from_signing_key(name, seed, signing_key, tier, ledger, snowball_params)
    }

    pub fn from_signing_key(
        name: impl Into<String>,
        seed: u64,
        signing_key: SigningKey,
        tier: HardwareTier,
        ledger: UtxoDagLedger,
        snowball_params: SnowballParams,
    ) -> Result<Self, AgentError> {
        let pubkey = signing_key.verifying_key().to_bytes();
        let profile = HardwareProfile::for_tier(tier);
        let device = HardwareProfile::select_compute_device();
        let on_chain = ledger.balance_of(&pubkey);

        let mut bank = AgentMetabolicBank::new(on_chain);
        let base_stem = CellDna::genesis_stem(seed);

        let initial_roles = [
            CellRole::Stem,
            CellRole::Stem,
            CellRole::Epidermal,
            CellRole::Vascular,
            CellRole::Visceral,
            CellRole::Neural,
        ];
        let seed_cell_energy =
            NanoSkn::from_nano((profile.mitosis_threshold.0 * 6 / 5).min(on_chain.0 / 20));

        let mut cells = Vec::new();
        let mut next_cell_id = 1u64;
        for (i, &role) in initial_roles.iter().enumerate() {
            if bank.blood_pool >= seed_cell_energy && seed_cell_energy.0 > 0 {
                let drawn = bank.draw_from_blood(seed_cell_energy)?;
                let cell_dna = CellDna::genesis_stem(seed.wrapping_add(i as u64 * 31));
                let dna = if role == CellRole::Stem {
                    cell_dna
                } else {
                    cell_dna.differentiate(role)
                };
                cells.push(MicroCell::spawn(next_cell_id, dna, drawn)?);
                next_cell_id += 1;
            }
        }

        // Preserve `base_stem` in cell 0 so lineage root is deterministic
        if let Some(first) = cells.first_mut() {
            first.dna = base_stem;
        }

        let utreexo_roots = ledger.build_utreexo_forest().roots;
        let (max_intra, max_k) = match tier {
            HardwareTier::TinyPeripheral => (10, 2),
            HardwareTier::StandardNode => (24, 3),
            HardwareTier::ColossalOrganism => (48, 4),
        };
        let peer_table = SynapticPeerTable::new(&pubkey, 0, 0, max_intra, max_k);

        let node = Self {
            name: name.into(),
            signing_key,
            pubkey,
            ownership_cert: None,
            profile,
            device,
            cluster_prefix: 0,
            prefix_len: 0,
            epoch: 0,
            pulse_seq: 0,
            cells,
            bank,
            ledger,
            utreexo_roots,
            voucher_channels: HashMap::new(),
            total_voucher_nano_earned: NanoSkn::ZERO,
            peer_table,
            snowball: SnowballEngine::new(snowball_params),
            next_cell_id,
            rng: ChaCha8Rng::seed_from_u64(seed ^ 0xA5A5_5A5A),
            total_mitosis_events: 0,
            total_apoptosis_events: 0,
            udp_sent: 0,
            udp_received: 0,
            udp_dropped_simulated: 0,
            fountain_decoders: HashMap::new(),
            imported_plasmids: 0,
            pending_snowball_queries: HashMap::new(),
            cluster_mitosis_votes: HashMap::new(),
        };
        node.verify_internal_skn_conservation()?;
        Ok(node)
    }

    /// Restores a previously persisted `AgentNode` from saved biological cells and bank state.
    pub fn restore_organism(opts: RestoreOrganismOptions) -> Result<Self, AgentError> {
        let pubkey = opts.signing_key.verifying_key().to_bytes();
        let profile = HardwareProfile::for_tier(opts.tier);
        let device = HardwareProfile::select_compute_device();
        let utreexo_roots = opts.ledger.build_utreexo_forest().roots;
        let (max_intra, max_k) = match opts.tier {
            HardwareTier::TinyPeripheral => (10, 2),
            HardwareTier::StandardNode => (24, 3),
            HardwareTier::ColossalOrganism => (48, 4),
        };
        let peer_table = SynapticPeerTable::new(&pubkey, 0, 0, max_intra, max_k);
        let max_id = opts.cells.iter().map(|c| c.cell_id).max().unwrap_or(0);

        let node = Self {
            name: opts.name,
            signing_key: opts.signing_key,
            pubkey,
            ownership_cert: None,
            profile,
            device,
            cluster_prefix: 0,
            prefix_len: 0,
            epoch: 0,
            pulse_seq: 0,
            cells: opts.cells,
            bank: opts.bank,
            ledger: opts.ledger,
            utreexo_roots,
            voucher_channels: HashMap::new(),
            total_voucher_nano_earned: NanoSkn::ZERO,
            peer_table,
            snowball: SnowballEngine::new(opts.snowball_params),
            next_cell_id: max_id + 1,
            rng: ChaCha8Rng::seed_from_u64(opts.seed ^ 0xA5A5_5A5A),
            total_mitosis_events: 0,
            total_apoptosis_events: 0,
            udp_sent: 0,
            udp_received: 0,
            udp_dropped_simulated: 0,
            fountain_decoders: HashMap::new(),
            imported_plasmids: 0,
            pending_snowball_queries: HashMap::new(),
            cluster_mitosis_votes: HashMap::new(),
        };
        node.verify_internal_skn_conservation()?;
        Ok(node)
    }

    /// Synchronizes internal `BloodPool` whenever on-chain `SKN` UTXOs arrive or are spent.
    pub fn sync_blood_with_ledger(&mut self) -> Result<(), AgentError> {
        let on_chain = self.ledger.balance_of(&self.pubkey);
        let mut cell_sum = NanoSkn::ZERO;
        for c in &self.cells {
            cell_sum = cell_sum.checked_add(c.energy)?;
        }
        if on_chain >= cell_sum {
            self.bank.blood_pool = on_chain.checked_sub(cell_sum)?;
        } else {
            let mut deficit = cell_sum.checked_sub(on_chain)?.0;
            self.bank.blood_pool = NanoSkn::ZERO;
            for c in self.cells.iter_mut().rev() {
                if deficit == 0 {
                    break;
                }
                let take = deficit.min(c.energy.0);
                c.energy = NanoSkn(c.energy.0 - take);
                deficit -= take;
            }
        }
        self.verify_internal_skn_conservation()
    }

    /// Asserts exact `NanoSkn` conservation: `on_chain_balance == blood_pool + sum(cells)`.
    pub fn verify_internal_skn_conservation(&self) -> Result<(), AgentError> {
        let expected = self.ledger.balance_of(&self.pubkey);
        self.bank
            .verify_invariant(expected, self.cells.iter().map(|c| c.energy))?;
        Ok(())
    }

    /// Executes 1 full multicellular biological tick on `candle` (Metabolism -> Batched AI Inference
    /// -> Competitive Darwinian SKN Allocation -> Stem Differentiation -> `1 -> 2` Mitosis -> Apoptosis).
    pub fn run_biological_tick(
        &mut self,
        stimulus: &[f32; DEFAULT_INPUT_DIM],
        target_signal: &[f32; 4],
    ) -> Result<Vec<f32>, AgentError> {
        self.pulse_seq = self.pulse_seq.wrapping_add(1);

        // 1. Basal Metabolism: cells burn ATP back into internal circulation (zero leakage)
        for cell in &mut self.cells {
            let burned = cell.metabolize_tick(self.profile.basal_burn_per_tick);
            self.bank.deposit_to_blood(burned)?;
        }

        // 2. Batched Candle Tensor Execution across ALL living cells simultaneously
        let batch_inputs: Vec<Vec<f32>> = (0..self.cells.len())
            .map(|_| stimulus.to_vec())
            .collect();
        let outputs = TissueBatch::forward_slice(&mut self.cells, &batch_inputs, &self.device)?;

        // 3. Evaluate micro-model accuracy & apply intra-tissue (per-organ) Darwinian selection
        let mut qualities = Vec::with_capacity(outputs.len());
        let mut role_sum_q: HashMap<CellRole, (f32, usize)> = HashMap::new();
        let mut cortical_sum = vec![0.0f32; 4];
        let mut cortical_cnt = 0usize;

        for (cell, out) in self.cells.iter().zip(outputs.iter()) {
            if cell.role() == CellRole::Neural {
                for k in 0..4 {
                    cortical_sum[k] += out[k];
                }
                cortical_cnt += 1;
            }
            let err: f32 = out
                .iter()
                .zip(target_signal.iter())
                .map(|(y, t)| (y - t).abs())
                .sum::<f32>()
                / 4.0;
            let q = (1.0 - err).clamp(0.0, 1.0);
            let entry = role_sum_q.entry(cell.role()).or_insert((0.0, 0));
            entry.0 += q;
            entry.1 += 1;
            qualities.push(q);
        }

        for (cell, &quality) in self.cells.iter_mut().zip(qualities.iter()) {
            let (sum_q, cnt_q) = role_sum_q.get(&cell.role()).copied().unwrap_or((0.0, 1));
            let organ_mean = sum_q / (cnt_q.max(1) as f32);
            // Only cells competitive within their own organ earn nutrient SKN; weak mutants decay
            let is_competitive = cnt_q <= 2 || quality >= organ_mean * 0.98;

            if cell.role() == CellRole::Stem || is_competitive {
                let desired_reward = NanoSkn::from_nano(
                    (quality * 320.0) as u128 + self.profile.basal_burn_per_tick.0,
                );
                let available = desired_reward.0.min(self.bank.blood_pool.0 / 8);
                if available > 0 {
                    let reward = self.bank.draw_from_blood(NanoSkn(available))?;
                    cell.absorb_energy(reward, quality)?;
                }
            } else {
                cell.health = (cell.health - 0.35).max(0.0);
            }
        }

        if cortical_cnt > 0 {
            for v in &mut cortical_sum {
                *v /= cortical_cnt as f32;
            }
        }

        // 4. Homeostatic Stem Cell Differentiation
        let missing_role = self.find_most_needed_role();
        if let Some(role) = missing_role {
            let stem_count = self
                .cells
                .iter()
                .filter(|c| c.role() == CellRole::Stem)
                .count();
            if stem_count > 1 {
                if let Some(stem_cell) = self
                    .cells
                    .iter_mut()
                    .find(|c| c.role() == CellRole::Stem)
                {
                    stem_cell.differentiate(role);
                }
            }
        }

        // 5. Mitosis (`1 -> 2` division + neuro-evolutionary mutation) up to hardware capacity
        let mut newborn_cells = Vec::new();
        let mut slots_left = self.profile.max_cells.saturating_sub(self.cells.len());

        for cell in &mut self.cells {
            if slots_left == 0 {
                break;
            }
            if cell.can_divide(&self.profile) {
                let child_id = self.next_cell_id;
                self.next_cell_id += 1;
                let (mut daughter, blood_fee) =
                    cell.divide_mitosis(child_id, &self.profile, &mut self.rng)?;
                self.bank.deposit_to_blood(blood_fee)?;

                if daughter.role() == CellRole::Stem {
                    if let Some(needed) = missing_role {
                        daughter.differentiate(needed);
                    }
                }
                newborn_cells.push(daughter);
                self.total_mitosis_events += 1;
                slots_left -= 1;
            }
        }
        self.cells.extend(newborn_cells);

        // 6. Programmed Apoptosis: eliminate low-fitness or starved cells while preserving >=1 cell per vital organ
        let mut surviving = Vec::with_capacity(self.cells.len());
        let mut kept_per_role: HashMap<CellRole, usize> = HashMap::new();
        for cell in self.cells.drain(..) {
            let role = cell.role();
            let already_kept = kept_per_role.get(&role).copied().unwrap_or(0);
            if already_kept == 0 {
                kept_per_role.insert(role, 1);
                surviving.push(cell);
                continue;
            }
            if cell.should_apoptose(&self.profile) {
                self.bank.deposit_to_blood(cell.energy)?;
                self.total_apoptosis_events += 1;
            } else {
                *kept_per_role.entry(role).or_default() += 1;
                surviving.push(cell);
            }
        }
        self.cells = surviving;

        self.verify_internal_skn_conservation()?;
        Ok(cortical_sum)
    }

    fn find_most_needed_role(&self) -> Option<CellRole> {
        let mut counts: HashMap<CellRole, usize> = HashMap::new();
        for c in &self.cells {
            *counts.entry(c.role()).or_default() += 1;
        }
        CellRole::ALL_SPECIALIZED
            .into_iter()
            .min_by_key(|r| counts.get(r).copied().unwrap_or(0))
    }

    /// Epidermal Immune Gate: uses `Epidermal` AI cells on `candle` to score an incoming UDP packet.
    pub fn epidermal_inspect_packet(
        &mut self,
        raw_len: usize,
        pkt: &ImpulsePacket,
    ) -> Result<bool, AgentError> {
        let len_norm = (raw_len as f32) / (MAX_UDP_DATAGRAM_BYTES as f32);
        let ttl_norm = (pkt.hop_ttl as f32) / 16.0;
        let epoch_diff = ((pkt.epoch as i64 - self.epoch as i64).abs() as f32 / 10.0).min(1.0);
        let features = [len_norm, ttl_norm, 1.0 - epoch_diff, 0.5, 0.2, 0.8, 0.1, 0.4];

        if let Some(skin_cell) = self
            .cells
            .iter_mut()
            .find(|c| c.role() == CellRole::Epidermal)
        {
            let score = skin_cell.forward(&features, &self.device)?;
            Ok(pkt.hop_ttl > 0 && score[0].is_finite())
        } else {
            Ok(pkt.hop_ttl > 0)
        }
    }

    /// Dynamic Cluster Mitosis (`0bP -> 0bP0` or `0bP1`) when cluster population exceeds threshold.
    pub fn split_cluster_prefix(&mut self) {
        if self.prefix_len < 30 {
            let bit_idx = self.prefix_len as usize;
            let byte = self.pubkey[bit_idx / 8];
            let bit = ((byte >> (7 - (bit_idx % 8))) & 1) as u32;
            self.cluster_prefix = (self.cluster_prefix << 1) | bit;
            self.prefix_len += 1;
            self.peer_table.local_prefix = self.cluster_prefix;
            self.peer_table.local_prefix_len = self.prefix_len;
        }
    }

    pub fn observe_peer_contact(&mut self, contact: PeerContact) {
        self.peer_table.observe_peer(contact);
    }

    /// Exports the highest-fitness cell's `CellDna` as Fountain-coded UDP impulses for Plasmid sharing.
    pub fn encode_elite_plasmid_droplets(
        &mut self,
        droplet_count: usize,
    ) -> Result<Vec<ImpulsePacket>, AgentError> {
        let elite_dna = self
            .cells
            .iter()
            .max_by(|a, b| {
                a.cumulative_fitness
                    .partial_cmp(&b.cumulative_fitness)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|c| c.dna.clone())
            .unwrap_or_else(|| CellDna::genesis_stem(1));

        let raw = bincode::serialize(&elite_dna).map_err(WireError::Codec)?;
        let mut enc = FountainEncoder::new(&raw)?;
        let mut packets = Vec::with_capacity(droplet_count);
        for _ in 0..droplet_count {
            let droplet = enc.next_droplet();
            self.pulse_seq = self.pulse_seq.wrapping_add(1);
            packets.push(ImpulsePacket::new_signed(
                &self.signing_key,
                self.cluster_prefix,
                self.prefix_len,
                4,
                self.epoch,
                self.pulse_seq,
                ImpulseBody::FountainDroplet(droplet),
            )?);
        }
        Ok(packets)
    }

    /// Binds a standalone UDP socket on SkyNet's default port (`0.0.0.0:8888`).
    pub async fn bind_default_udp() -> Result<UdpSocket, AgentError> {
        Ok(UdpSocket::bind(("0.0.0.0", DEFAULT_PORT)).await?)
    }

    /// Sends an `ImpulsePacket` over a real `UdpSocket`, simulating ambient lossy network drops.
    pub async fn send_lossy_udp(
        &mut self,
        socket: &UdpSocket,
        target: SocketAddr,
        pkt: &ImpulsePacket,
        loss_probability: f64,
    ) -> Result<bool, AgentError> {
        let bytes = pkt.encode_datagram()?;
        if loss_probability > 0.0 && self.rng.gen_bool(loss_probability.clamp(0.0, 1.0)) {
            self.udp_dropped_simulated += 1;
            return Ok(false);
        }
        socket.send_to(&bytes, target).await?;
        self.udp_sent += 1;
        Ok(true)
    }

    /// Processes an incoming UDP datagram (`SnowballPull`, `SnowballReply`, `GossipTx`, `FountainDroplet`).
    pub fn handle_incoming_datagram(
        &mut self,
        raw: &[u8],
    ) -> Result<Option<ImpulsePacket>, AgentError> {
        let pkt = ImpulsePacket::decode_and_verify(raw)?;
        if !self.epidermal_inspect_packet(raw.len(), &pkt)? {
            return Ok(None);
        }
        self.udp_received += 1;

        match pkt.body {
            ImpulseBody::Heartbeat { .. } => Ok(None),
            ImpulseBody::GossipTx(tx) => {
                if self.ledger.admit_pending_tx(tx.clone()).is_ok() {
                    self.snowball.register_tx(&tx);
                }
                Ok(None)
            }
            ImpulseBody::WitnessedTx {
                tx,
                spent_outputs,
                proofs,
            } => {
                if tx.inputs.len() == spent_outputs.len() && tx.inputs.len() == proofs.len() {
                    for ((inp, out), proof) in tx
                        .inputs
                        .iter()
                        .zip(spent_outputs.iter())
                        .zip(proofs.iter())
                    {
                        self.utreexo_roots.verify_utxo(inp, out, proof)?;
                    }
                    if self.ledger.admit_pending_tx(tx.clone()).is_ok() {
                        self.snowball.register_tx(&tx);
                    }
                }
                Ok(None)
            }
            ImpulseBody::SynapticMicroPay { voucher, stimulus } => {
                if voucher.payee_pubkey == self.pubkey {
                    if let Some(utxo) = self.ledger.get_utxo(&voucher.bond_utxo) {
                        if let CustodyPolicy::AgentBond {
                            max_voucher_allowance,
                            ..
                        } = utxo.policy
                        {
                            let tracker = self
                                .voucher_channels
                                .entry(voucher.bond_utxo)
                                .or_insert_with(|| {
                                    VoucherChannelTracker::new(
                                        voucher.bond_utxo,
                                        max_voucher_allowance,
                                    )
                                });
                            if let Ok(earned_delta) = tracker.receive_udp_voucher(voucher) {
                                self.total_voucher_nano_earned = self
                                    .total_voucher_nano_earned
                                    .checked_add(earned_delta)?;
                                if let Some(brain_cell) = self
                                    .cells
                                    .iter_mut()
                                    .find(|c| c.role() == CellRole::Neural)
                                {
                                    let _ = brain_cell.forward(&stimulus, &self.device)?;
                                }
                            }
                        }
                    }
                }
                Ok(None)
            }
            ImpulseBody::SnowballPull {
                query_id,
                contested_utxos,
            } => {
                let mut votes = Vec::new();
                for u in contested_utxos {
                    if let Some(vote) = self.snowball.vote_for(&u) {
                        votes.push((u, vote));
                    }
                }
                self.pulse_seq = self.pulse_seq.wrapping_add(1);
                let reply = ImpulsePacket::new_signed(
                    &self.signing_key,
                    self.cluster_prefix,
                    self.prefix_len,
                    2,
                    self.epoch,
                    self.pulse_seq,
                    ImpulseBody::SnowballReply { query_id, votes },
                )?;
                Ok(Some(reply))
            }
            ImpulseBody::SnowballReply { query_id, votes } => {
                // Mitigates SEC-01: Drop unsolicited or replay Snowball replies
                if let Some((_, queried_peers, contested)) =
                    self.pending_snowball_queries.get_mut(&query_id)
                {
                    if queried_peers.remove(&pkt.sender_pubkey) {
                        for (u, chosen_tx) in votes {
                            if contested.contains(&u) {
                                self.snowball.record_incoming_vote(u, chosen_tx);
                            }
                        }
                    }
                }
                Ok(None)
            }
            ImpulseBody::ClusterMitosis {
                parent_prefix,
                parent_prefix_len,
                ..
            } => {
                // Mitigates SEC-09: Cluster mitosis requires quorum consensus from known intra-cluster peers
                if self.cluster_prefix == parent_prefix && self.prefix_len == parent_prefix_len {
                    let is_known = self
                        .peer_table
                        .intra_cluster_peers
                        .iter()
                        .any(|p| p.pubkey == pkt.sender_pubkey);
                    if is_known {
                        let votes = self
                            .cluster_mitosis_votes
                            .entry((parent_prefix, parent_prefix_len))
                            .or_default();
                        votes.insert(pkt.sender_pubkey);
                        if votes.len() >= 2 || self.cells.len() >= self.profile.max_cells * 9 / 10 {
                            self.split_cluster_prefix();
                            self.cluster_mitosis_votes.remove(&(parent_prefix, parent_prefix_len));
                        }
                    }
                }
                Ok(None)
            }
            ImpulseBody::FountainDroplet(droplet) => {
                // Mitigates SEC-11: Bound concurrent in-flight fountain decoders to 32 to prevent OOM
                if !self.fountain_decoders.contains_key(&droplet.object_hash)
                    && self.fountain_decoders.len() >= 32
                {
                    return Ok(None);
                }
                let hash = droplet.object_hash;
                let dec = self
                    .fountain_decoders
                    .entry(hash)
                    .or_insert_with(|| FountainDecoder::from_first_droplet(&droplet));
                if let Some(bytes) = dec.absorb(droplet)? {
                    self.fountain_decoders.remove(&hash);
                    if let Ok(donor_dna) = bincode::deserialize::<CellDna>(&bytes) {
                        if donor_dna.verify_integrity().is_ok() {
                            // Mitigates SEC-08: Protect immune (Epidermal) and vital (Visceral) cells from unverified plasmids
                            if let Some(candidate) = self.cells.iter_mut().find(|c| {
                                c.role() != CellRole::Epidermal
                                    && c.role() != CellRole::Visceral
                                    && c.health < 0.95
                            }) {
                                if let Ok(crossed) =
                                    candidate.dna.horizontal_crossover(&donor_dna, &mut self.rng)
                                {
                                    candidate.dna = crossed;
                                    self.imported_plasmids += 1;
                                }
                            }
                        }
                    }
                }
                Ok(None)
            }
            ImpulseBody::Compute(_msg) => {
                // Handled via ComputeFlowCoordinator
                Ok(None)
            }
        }
    }

    /// Registers an active outgoing Snowball query for a set of sampled peers.
    pub fn initiate_snowball_query(
        &mut self,
        query_id: u64,
        queried_peers: std::collections::HashSet<[u8; 32]>,
        contested_utxos: Vec<UtxoId>,
    ) {
        self.pending_snowball_queries.insert(
            query_id,
            (self.epoch, queried_peers, contested_utxos),
        );
    }

    /// Closes the current Snowball UDP sampling window, commits any newly finalized UTXOs,
    /// decays synaptic routing table weights, and synchronizes internal cellular `BloodPool` with on-chain `SKN`.
    pub fn finalize_snowball_pulse(&mut self) -> Result<Vec<[u8; 32]>, AgentError> {
        self.peer_table.decay_synapses();
        self.pending_snowball_queries
            .retain(|_, (epoch, _, _)| self.epoch.saturating_sub(*epoch) <= 5);

        let winners = self.snowball.close_pulse_window();
        for tx_hash in &winners {
            if self.ledger.pending_tx(tx_hash).is_some() {
                self.ledger.finalize_tx(tx_hash)?;
                self.utreexo_roots = self.ledger.build_utreexo_forest().roots;
                self.sync_blood_with_ledger()?;
            }
        }
        Ok(winners)
    }

    pub fn sample_peers(&mut self, peers: &[SocketAddr]) -> Vec<SocketAddr> {
        let k = self.snowball.params.sample_k.min(peers.len());
        peers
            .choose_multiple(&mut self.rng, k)
            .copied()
            .collect()
    }

    pub fn stats(&self) -> OrganismStats {
        let mut role_counts = HashMap::new();
        let mut max_gen = 0u64;
        for c in &self.cells {
            *role_counts.entry(c.role()).or_default() += 1;
            max_gen = max_gen.max(c.dna.generation);
        }
        OrganismStats {
            name: self.name.clone(),
            tier: self.profile.tier,
            cluster_prefix: self.cluster_prefix,
            prefix_len: self.prefix_len,
            total_cells: self.cells.len(),
            role_counts,
            max_generation: max_gen,
            on_chain_balance: self.ledger.balance_of(&self.pubkey),
            blood_pool: self.bank.blood_pool,
            total_mitosis_events: self.total_mitosis_events,
            total_apoptosis_events: self.total_apoptosis_events,
            udp_sent: self.udp_sent,
            udp_received: self.udp_received,
            udp_dropped_simulated: self.udp_dropped_simulated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skynet_ledger::TxOutput;

    #[test]
    fn colossal_agent_grows_larger_than_tiny_peripheral_while_conserving_skn() {
        let mut rng = ChaCha8Rng::seed_from_u64(999);
        let k1 = SigningKey::generate(&mut rng);
        let k2 = SigningKey::generate(&mut rng);
        let alloc = NanoSkn::from_skn(10_000).unwrap();

        let (ledger, _) = UtxoDagLedger::from_genesis(vec![
            TxOutput::direct(k1.verifying_key().to_bytes(), alloc),
            TxOutput::direct(k2.verifying_key().to_bytes(), alloc),
        ])
        .unwrap();

        let mut giant = AgentNode::from_signing_key(
            "Giant",
            1,
            k1,
            HardwareTier::ColossalOrganism,
            ledger.clone(),
            SnowballParams::default(),
        )
        .unwrap();
        let mut tiny = AgentNode::from_signing_key(
            "Tiny",
            2,
            k2,
            HardwareTier::TinyPeripheral,
            ledger,
            SnowballParams::default(),
        )
        .unwrap();

        let stim = [0.5, -0.3, 0.7, 0.2, -0.6, 0.4, 0.1, -0.8];
        let target = [0.7, -0.6, 0.5, -0.4];
        for _ in 0..12 {
            giant.run_biological_tick(&stim, &target).unwrap();
            tiny.run_biological_tick(&stim, &target).unwrap();
        }

        assert!(tiny.cells.len() <= 32);
        assert!(giant.cells.len() > tiny.cells.len() * 3);
        assert!(giant.total_apoptosis_events > 0);
        giant.verify_internal_skn_conservation().unwrap();
        tiny.verify_internal_skn_conservation().unwrap();
    }

    #[test]
    fn test_unsolicited_snowball_reply_is_dropped() {
        let mut rng = ChaCha8Rng::seed_from_u64(1234);
        let node_key = SigningKey::generate(&mut rng);
        let attacker_key = SigningKey::generate(&mut rng);
        let alloc = NanoSkn::from_skn(100).unwrap();

        let (ledger, _) = UtxoDagLedger::from_genesis(vec![
            TxOutput::direct(node_key.verifying_key().to_bytes(), alloc),
        ])
        .unwrap();

        let mut node = AgentNode::from_signing_key(
            "Node",
            1,
            node_key,
            HardwareTier::StandardNode,
            ledger,
            SnowballParams::default(),
        )
        .unwrap();

        let fake_utxo = UtxoId {
            tx_hash: [1u8; 32],
            vout: 0,
        };
        let fake_vote = [2u8; 32];

        // Attacker sends an unsolicited SnowballReply with unqueried query_id = 999
        let unsolicited_reply = ImpulsePacket::new_signed(
            &attacker_key,
            node.cluster_prefix,
            node.prefix_len,
            2,
            node.epoch,
            1,
            ImpulseBody::SnowballReply {
                query_id: 999,
                votes: vec![(fake_utxo, fake_vote)],
            },
        )
        .unwrap();

        let raw = unsolicited_reply.encode_datagram().unwrap();
        node.handle_incoming_datagram(&raw).unwrap();

        // Must NOT have recorded the unsolicited vote!
        assert!(node.snowball.vote_for(&fake_utxo).is_none());
    }

    #[test]
    fn test_cluster_mitosis_requires_peer_quorum() {
        let mut rng = ChaCha8Rng::seed_from_u64(5678);
        let node_key = SigningKey::generate(&mut rng);
        let stranger_key = SigningKey::generate(&mut rng);
        let peer_key = SigningKey::generate(&mut rng);
        let alloc = NanoSkn::from_skn(100).unwrap();

        let (ledger, _) = UtxoDagLedger::from_genesis(vec![
            TxOutput::direct(node_key.verifying_key().to_bytes(), alloc),
        ])
        .unwrap();

        let mut node = AgentNode::from_signing_key(
            "Node",
            1,
            node_key,
            HardwareTier::StandardNode,
            ledger,
            SnowballParams::default(),
        )
        .unwrap();

        let initial_prefix_len = node.prefix_len;

        // Stranger tries to force mitosis without being a known peer
        let stranger_pkt = ImpulsePacket::new_signed(
            &stranger_key,
            node.cluster_prefix,
            node.prefix_len,
            2,
            node.epoch,
            1,
            ImpulseBody::ClusterMitosis {
                parent_prefix: node.cluster_prefix,
                parent_prefix_len: node.prefix_len,
                epoch: node.epoch,
            },
        )
        .unwrap();
        node.handle_incoming_datagram(&stranger_pkt.encode_datagram().unwrap()).unwrap();
        assert_eq!(node.prefix_len, initial_prefix_len, "stranger cannot force mitosis");

        // Register peer as an active intra-cluster peer
        node.peer_table.observe_peer(PeerContact {
            pubkey: peer_key.verifying_key().to_bytes(),
            addr: "10.0.0.1:8888".parse().unwrap(),
            cluster_prefix: node.cluster_prefix,
            prefix_len: node.prefix_len,
            bonded_skn_whole: 100,
            synaptic_score: 200,
        });

        // 1 peer vote is recorded but still insufficient for quorum (< 2)
        let peer_pkt1 = ImpulsePacket::new_signed(
            &peer_key,
            node.cluster_prefix,
            node.prefix_len,
            2,
            node.epoch,
            1,
            ImpulseBody::ClusterMitosis {
                parent_prefix: node.cluster_prefix,
                parent_prefix_len: node.prefix_len,
                epoch: node.epoch,
            },
        )
        .unwrap();
        node.handle_incoming_datagram(&peer_pkt1.encode_datagram().unwrap()).unwrap();
        assert_eq!(node.prefix_len, initial_prefix_len, "single peer cannot trigger mitosis without quorum");
    }
}
