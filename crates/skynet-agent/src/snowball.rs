//! `snowball`: Metastable Snowball consensus engine and `AgentNode` pulse finality over UDP.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::net::SocketAddr;

use rand::seq::SliceRandom;
use skynet_ledger::{SknTransaction, UtxoId};

use crate::organism_state::AgentNode;
use crate::AgentError;

/// Snowball consensus parameters tuned for lossy UDP sub-sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        Self { sample_k: 5, quorum_alpha: 3, finality_beta: 5 }
    }
}

#[derive(Debug, Clone)]
struct ConflictPreference {
    preferred_tx: [u8; 32],
    last_round_winner: Option<[u8; 32]>,
    confidence: u16,
}

/// Tracks metastable Snowball convergence across contested UTXOs without TCP retransmissions.
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
            self.preferences.entry(*inp).or_insert(ConflictPreference {
                preferred_tx: tx_hash,
                last_round_winner: None,
                confidence: 0,
            });
        }
    }

    pub fn contested_utxos(&self) -> Vec<UtxoId> {
        self.preferences.keys().copied().collect()
    }

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
        if !self.preferences.contains_key(&utxo) {
            return;
        }
        self.active_round_votes.entry(utxo).or_default().push(voted_tx);
    }

    /// Closes the current UDP pulse window and returns newly finalized transaction hashes.
    pub fn close_pulse_window(&mut self) -> Vec<[u8; 32]> {
        let collected = std::mem::take(&mut self.active_round_votes);
        let mut newly_finalized = Vec::new();
        for (utxo, votes) in collected {
            let Some(winner) = self.observe_sample_tally(utxo, &votes) else {
                continue;
            };
            if !newly_finalized.contains(&winner) {
                newly_finalized.push(winner);
            }
        }
        newly_finalized
    }

    /// Evaluates surviving UDP sample replies for `utxo` (ignoring dropped UDP packets).
    pub fn observe_sample_tally(&mut self, utxo: UtxoId, votes: &[[u8; 32]]) -> Option<[u8; 32]> {
        let pref = self.preferences.get_mut(&utxo)?;
        let mut counts: HashMap<[u8; 32], usize> = HashMap::new();
        for v in votes {
            *counts.entry(*v).or_default() += 1;
        }

        let (winner, _) = counts
            .into_iter()
            .max_by_key(|(_, cnt)| *cnt)
            .filter(|(_, cnt)| *cnt >= self.params.quorum_alpha)?;

        pref.preferred_tx = winner;
        if pref.last_round_winner == Some(winner) {
            pref.confidence = pref.confidence.saturating_add(1);
        } else {
            pref.last_round_winner = Some(winner);
            pref.confidence = 1;
        }
        if pref.confidence < self.params.finality_beta {
            return None;
        }
        self.preferences.remove(&utxo);
        self.decided.insert(utxo, winner);
        Some(winner)
    }
}

impl AgentNode {
    /// Registers an active outgoing Snowball query for a set of sampled peers.
    pub fn initiate_snowball_query(
        &mut self,
        query_id: u64,
        queried_peers: HashSet<[u8; 32]>,
        contested_utxos: Vec<UtxoId>,
    ) {
        self.pending_snowball_queries
            .insert(query_id, (self.epoch, queried_peers, contested_utxos));
    }

    /// Closes the current Snowball UDP sampling window, commits newly finalized UTXOs,
    /// decays synaptic weights, and synchronizes internal cellular `BloodPool` with on-chain `SKN`.
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
        peers.choose_multiple(&mut self.rng, k).copied().collect()
    }
}
