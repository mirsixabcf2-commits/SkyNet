//! `datagram_router`: Lossy UDP socket I/O, epidermal immune inspection, Snowball reply
//! routing, and live `ImpulseBody::{Compute, Rendezvous, MicroTrain}` dispatch on `AgentNode`.

use std::net::SocketAddr;

use rand::Rng;
use skynet_cell::{compute_candle::CandleTaskInput, HardwareTier};
use skynet_dna::{CellDna, CellRole};
use skynet_ledger::{CustodyPolicy, NanoSkn, SknTransaction, SynapticVoucher, TxOutput, UtreexoProof, UtxoId, VoucherChannelTracker};
use skynet_wire::{ComputeMessage, FountainDecoder, FountainDroplet, FountainEncoder, ImpulseBody, ImpulsePacket, MicroTrainWireMessage, PeerContact, RendezvousMessage, WireError, DEFAULT_PORT, MAX_UDP_DATAGRAM_BYTES};
use tokio::net::UdpSocket;

use crate::auditor_flow::AuditorCoordinator;
use crate::compute_flow::ComputeFlowCoordinator;
use crate::opportunistic_worker::{OpportunisticWorkerEngine, OpportunisticWorkerError, ShardExecutionStep};
use crate::organism_state::AgentNode;
use crate::workload_mux::WorkloadClass;
use crate::AgentError;

impl AgentNode {
    pub fn epidermal_inspect_packet(&mut self, raw_len: usize, pkt: &ImpulsePacket) -> Result<bool, AgentError> {
        let len_norm = (raw_len as f32) / (MAX_UDP_DATAGRAM_BYTES as f32);
        let ttl_norm = (pkt.hop_ttl as f32) / 16.0;
        let epoch_diff = ((pkt.epoch as i64 - self.epoch as i64).abs() as f32 / 10.0).min(1.0);
        let features = [len_norm, ttl_norm, 1.0 - epoch_diff, 0.5, 0.2, 0.8, 0.1, 0.4];
        let Some(skin) = self.cells.iter_mut().find(|c| c.role() == CellRole::Epidermal) else { return Ok(pkt.hop_ttl > 0); };
        Ok(pkt.hop_ttl > 0 && skin.forward(&features, &self.device)?[0].is_finite())
    }

    pub fn encode_elite_plasmid_droplets(&mut self, droplet_count: usize) -> Result<Vec<ImpulsePacket>, AgentError> {
        let elite = self.cells.iter()
            .max_by(|a, b| a.cumulative_fitness.partial_cmp(&b.cumulative_fitness).unwrap_or(std::cmp::Ordering::Equal))
            .map(|c| c.dna.clone()).unwrap_or_else(|| CellDna::genesis_stem(1));
        let mut enc = FountainEncoder::new(&bincode::serialize(&elite).map_err(WireError::Codec)?)?;
        (0..droplet_count).map(|_| self.sign_reply(4, ImpulseBody::FountainDroplet(enc.next_droplet()))).collect()
    }

    pub async fn bind_default_udp() -> Result<UdpSocket, AgentError> {
        Ok(UdpSocket::bind(("0.0.0.0", DEFAULT_PORT)).await?)
    }

    pub async fn send_lossy_udp(&mut self, socket: &UdpSocket, target: SocketAddr, pkt: &ImpulsePacket, loss_probability: f64) -> Result<bool, AgentError> {
        let bytes = pkt.encode_datagram()?;
        if loss_probability > 0.0 && self.rng.gen_bool(loss_probability.clamp(0.0, 1.0)) {
            self.udp_dropped_simulated += 1;
            return Ok(false);
        }
        socket.send_to(&bytes, target).await?;
        self.udp_sent += 1;
        Ok(true)
    }

    pub fn handle_incoming_datagram(&mut self, raw: &[u8]) -> Result<Option<ImpulsePacket>, AgentError> {
        self.handle_incoming_datagram_from(raw, None)
    }

    pub fn handle_incoming_datagram_from(&mut self, raw: &[u8], src_addr: Option<SocketAddr>) -> Result<Option<ImpulsePacket>, AgentError> {
        let pkt = ImpulsePacket::decode_and_verify(raw)?;
        if !self.epidermal_inspect_packet(raw.len(), &pkt)? { return Ok(None); }
        self.udp_received += 1;
        let sender = pkt.sender_pubkey;
        match pkt.body {
            ImpulseBody::Heartbeat { .. } => Ok(None),
            ImpulseBody::GossipTx(tx) => { if self.ledger.admit_pending_tx(tx.clone()).is_ok() { self.snowball.register_tx(&tx); } Ok(None) }
            ImpulseBody::WitnessedTx { tx, spent_outputs, proofs } => self.on_witnessed_tx(tx, &spent_outputs, &proofs),
            ImpulseBody::SynapticMicroPay { voucher, stimulus } => self.on_synaptic_micropay(voucher, &stimulus),
            ImpulseBody::SnowballPull { query_id, contested_utxos } => self.on_snowball_pull(query_id, contested_utxos),
            ImpulseBody::SnowballReply { query_id, votes } => self.on_snowball_reply(sender, query_id, votes),
            ImpulseBody::ClusterMitosis { parent_prefix, parent_prefix_len, .. } => self.on_cluster_mitosis(sender, parent_prefix, parent_prefix_len),
            ImpulseBody::FountainDroplet(droplet) => self.on_fountain_droplet(droplet),
            ImpulseBody::Compute(msg) => self.on_compute_msg(msg),
            ImpulseBody::Rendezvous(msg) => self.on_rendezvous_msg(msg, src_addr),
            ImpulseBody::MicroTrain(msg) => self.on_micro_train_msg(msg),
        }
    }

    fn sign_reply(&mut self, hop_ttl: u8, body: ImpulseBody) -> Result<ImpulsePacket, AgentError> {
        self.pulse_seq = self.pulse_seq.wrapping_add(1);
        Ok(ImpulsePacket::new_signed(&self.signing_key, self.cluster_prefix, self.prefix_len, hop_ttl, self.epoch, self.pulse_seq, body)?)
    }

    fn on_witnessed_tx(&mut self, tx: SknTransaction, outs: &[TxOutput], proofs: &[UtreexoProof]) -> Result<Option<ImpulsePacket>, AgentError> {
        if tx.inputs.len() != outs.len() || tx.inputs.len() != proofs.len() { return Ok(None); }
        for ((inp, out), proof) in tx.inputs.iter().zip(outs.iter()).zip(proofs.iter()) { self.utreexo_roots.verify_utxo(inp, out, proof)?; }
        if self.ledger.admit_pending_tx(tx.clone()).is_ok() { self.snowball.register_tx(&tx); }
        Ok(None)
    }

    fn on_synaptic_micropay(&mut self, voucher: SynapticVoucher, stimulus: &[f32; 8]) -> Result<Option<ImpulsePacket>, AgentError> {
        if voucher.payee_pubkey != self.pubkey { return Ok(None); }
        let Some(&CustodyPolicy::AgentBond { max_voucher_allowance, .. }) = self.ledger.get_utxo(&voucher.bond_utxo).map(|u| &u.policy) else { return Ok(None); };
        let tracker = self.voucher_channels.entry(voucher.bond_utxo).or_insert_with(|| VoucherChannelTracker::new(voucher.bond_utxo, max_voucher_allowance));
        if let Ok(delta) = tracker.receive_udp_voucher(voucher) {
            self.total_voucher_nano_earned = self.total_voucher_nano_earned.checked_add(delta)?;
            if let Some(brain) = self.cells.iter_mut().find(|c| c.role() == CellRole::Neural) { let _ = brain.forward(stimulus, &self.device)?; }
        }
        Ok(None)
    }

    fn on_snowball_pull(&mut self, query_id: u64, contested: Vec<UtxoId>) -> Result<Option<ImpulsePacket>, AgentError> {
        let votes = contested.into_iter().filter_map(|u| self.snowball.vote_for(&u).map(|v| (u, v))).collect();
        Ok(Some(self.sign_reply(2, ImpulseBody::SnowballReply { query_id, votes })?))
    }

    fn on_snowball_reply(&mut self, sender: [u8; 32], query_id: u64, votes: Vec<(UtxoId, [u8; 32])>) -> Result<Option<ImpulsePacket>, AgentError> {
        let Some((_, peers, contested)) = self.pending_snowball_queries.get_mut(&query_id) else { return Ok(None); };
        if !peers.remove(&sender) { return Ok(None); }
        for (u, tx) in votes { if contested.contains(&u) { self.snowball.record_incoming_vote(u, tx); } }
        Ok(None)
    }

    fn on_cluster_mitosis(&mut self, sender: [u8; 32], p_pref: u32, p_len: u8) -> Result<Option<ImpulsePacket>, AgentError> {
        if self.cluster_prefix != p_pref || self.prefix_len != p_len { return Ok(None); }
        if !self.peer_table.intra_cluster_peers.iter().any(|p| p.pubkey == sender) { return Ok(None); }
        let votes = self.cluster_mitosis_votes.entry((p_pref, p_len)).or_default();
        votes.insert(sender);
        if votes.len() >= 2 || self.cells.len() >= self.profile.max_cells * 9 / 10 {
            self.split_cluster_prefix();
            self.cluster_mitosis_votes.remove(&(p_pref, p_len));
        }
        Ok(None)
    }

    fn on_fountain_droplet(&mut self, droplet: FountainDroplet) -> Result<Option<ImpulsePacket>, AgentError> {
        if self.opportunistic_worker.has_pending_shard(&droplet.object_hash) {
            let Some((m, batch)) = self.opportunistic_worker.absorb_shard_droplet(droplet)? else { return Ok(None); };
            let cfg = OpportunisticWorkerEngine::config_from_manifest(&m);
            if let ShardExecutionStep::Completed { claim, .. } = self.opportunistic_worker.run_or_pause_shard(&self.signing_key, &cfg, &batch)? {
                return Ok(Some(self.sign_reply(4, ImpulseBody::MicroTrain(MicroTrainWireMessage::WeightDeltaClaim(*claim)))?));
            }
            return Ok(None);
        }
        if !self.fountain_decoders.contains_key(&droplet.object_hash) && self.fountain_decoders.len() >= 32 { return Ok(None); }
        let hash = droplet.object_hash;
        let Some(bytes) = self.fountain_decoders.entry(hash).or_insert_with(|| FountainDecoder::from_first_droplet(&droplet)).absorb(droplet)? else { return Ok(None); };
        self.fountain_decoders.remove(&hash);
        if let Ok(donor) = bincode::deserialize::<CellDna>(&bytes) { if donor.verify_integrity().is_ok() { self.apply_plasmid_crossover(&donor); } }
        Ok(None)
    }

    fn apply_plasmid_crossover(&mut self, donor: &CellDna) {
        let Some(cand) = self.cells.iter_mut().find(|c| c.role() != CellRole::Epidermal && c.role() != CellRole::Visceral && c.health < 0.95) else { return; };
        if let Ok(crossed) = cand.dna.horizontal_crossover(donor, &mut self.rng) { cand.dna = crossed; self.imported_plasmids += 1; }
    }

    fn on_compute_msg(&mut self, msg: ComputeMessage) -> Result<Option<ImpulsePacket>, AgentError> {
        match msg {
            ComputeMessage::TaskAnnouncement(m) => {
                self.opportunistic_worker.mux.ensure_admitted(self.opportunistic_worker.governor_state(), WorkloadClass::FastInferenceSub10ms).map_err(OpportunisticWorkerError::Mux)?;
                self.opportunistic_worker.power_guard.evaluate_bid(NanoSkn(m.bounty_nano_skn), 10).map_err(OpportunisticWorkerError::Mux)?;
                let tier = match self.profile.tier { HardwareTier::TinyPeripheral => 1, HardwareTier::StandardNode => 2, HardwareTier::ColossalOrganism => 3 };
                let bid = ComputeFlowCoordinator::create_bid(&self.signing_key, &m, tier, NanoSkn(m.min_worker_bond_nano_skn))?;
                Ok(Some(self.sign_reply(2, ImpulseBody::Compute(ComputeMessage::TaskBid(bid)))?))
            }
            ComputeMessage::DirectPayload { is_result: false, data, .. } => {
                let input: CandleTaskInput = bincode::deserialize(&data).map_err(WireError::Codec)?;
                let (_, claim) = self.opportunistic_worker.execute_fast_inference(&self.signing_key, &input)?;
                Ok(Some(self.sign_reply(2, ImpulseBody::Compute(ComputeMessage::CompletionClaim(claim)))?))
            }
            ComputeMessage::AuditorBid(bid) => { AuditorCoordinator::register_bid(&mut self.compute_coordinator.escrow_vault, &bid)?; Ok(None) }
            _ => Ok(None),
        }
    }

    fn on_rendezvous_msg(&mut self, msg: RendezvousMessage, src_addr: Option<SocketAddr>) -> Result<Option<ImpulsePacket>, AgentError> {
        let fallback = SocketAddr::from(([127, 0, 0, 1], DEFAULT_PORT));
        match msg {
            RendezvousMessage::PheromoneHello(h) => {
                let e = self.rendezvous.register_hello(&h, src_addr.unwrap_or(h.internal_addr), h.timestamp_sec)?;
                self.observe_peer_contact(PeerContact { pubkey: e.pubkey, addr: e.public_addr, cluster_prefix: 0, prefix_len: 0, bonded_skn_whole: 1, synaptic_score: 100 });
                Ok(None)
            }
            RendezvousMessage::PunchRendezvous(cmd) => {
                let (_, burst) = self.rendezvous.build_punch_probe_burst(self.pubkey, &cmd, self.epoch)?;
                let Some(first) = burst.into_iter().next() else { return Ok(None); };
                Ok(Some(self.sign_reply(2, ImpulseBody::Rendezvous(RendezvousMessage::PunchProbe(first)))?))
            }
            RendezvousMessage::PunchProbe(p) => {
                if let Some(e) = self.rendezvous.handle_punch_probe(&p, src_addr.unwrap_or(fallback), self.epoch) {
                    self.observe_peer_contact(PeerContact { pubkey: e.pubkey, addr: e.public_addr, cluster_prefix: 0, prefix_len: 0, bonded_skn_whole: 1, synaptic_score: 150 });
                }
                Ok(None)
            }
            RendezvousMessage::LanBeacon(b) => {
                let e = self.rendezvous.handle_lan_beacon(&b, src_addr.unwrap_or(fallback).ip(), self.epoch)?;
                self.observe_peer_contact(PeerContact { pubkey: e.pubkey, addr: e.public_addr, cluster_prefix: 0, prefix_len: 0, bonded_skn_whole: 1, synaptic_score: 200 });
                Ok(None)
            }
            RendezvousMessage::GangliaRelayFrame(f) if f.target_pubkey == self.pubkey => {
                let inner = ImpulsePacket::decode_and_verify(&f.inner_datagram)?;
                if matches!(inner.body, ImpulseBody::Rendezvous(RendezvousMessage::GangliaRelayFrame(_))) { return Ok(None); }
                self.handle_incoming_datagram_from(&f.inner_datagram, src_addr)
            }
            RendezvousMessage::GangliaRelayFrame(f) => { let _ = self.rendezvous.route_ganglia_relay(f, self.epoch)?; Ok(None) }
        }
    }

    fn on_micro_train_msg(&mut self, msg: MicroTrainWireMessage) -> Result<Option<ImpulsePacket>, AgentError> {
        match msg {
            MicroTrainWireMessage::MicroBatchManifest(m) => { self.opportunistic_worker.accept_manifest(m, 1_000)?; Ok(None) }
            MicroTrainWireMessage::WeightDeltaClaim(c) => {
                c.verify_signature()?;
                let Some(ref_res) = self.opportunistic_worker.canary_reference(&c.task_id, c.shard_id).cloned() else { return Ok(None); };
                let Some(rep) = self.opportunistic_worker.verify_claim_against_canary(&self.signing_key, &c, &ref_res)? else { return Ok(None); };
                let _ = OpportunisticWorkerEngine::apply_fraud_report_to_vault(&mut self.compute_coordinator.escrow_vault, &rep);
                Ok(Some(self.sign_reply(4, ImpulseBody::MicroTrain(MicroTrainWireMessage::CanaryFraudReport(rep)))?))
            }
            MicroTrainWireMessage::CanaryFraudReport(r) => { let _ = OpportunisticWorkerEngine::apply_fraud_report_to_vault(&mut self.compute_coordinator.escrow_vault, &r)?; Ok(None) }
            MicroTrainWireMessage::ChunkVoucherDelivery(d) => {
                if d.voucher.payee_pubkey != self.pubkey { return Ok(None); }
                let Some(&CustodyPolicy::AgentBond { max_voucher_allowance, .. }) = self.ledger.get_utxo(&d.voucher.bond_utxo).map(|u| &u.policy) else { return Ok(None); };
                self.total_voucher_nano_earned = self.total_voucher_nano_earned.checked_add(self.opportunistic_worker.receive_chunk_voucher(d, max_voucher_allowance)?)?;
                let Some(sweep_tx) = self.opportunistic_worker.try_build_yield_sweep(&self.signing_key, &self.ledger)? else { return Ok(None); };
                if self.ledger.admit_pending_tx(sweep_tx.clone()).is_ok() { self.snowball.register_tx(&sweep_tx); }
                Ok(Some(self.sign_reply(4, ImpulseBody::GossipTx(sweep_tx))?))
            }
        }
    }
}
