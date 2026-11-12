//! `runtime_p2p`: Real P2P socket driver enforcing `EpidermalFilter` (`GENESIS_HASH` + `±30s`),
//! `25s` `PheromoneHello` Seed keepalives, `< 1s` `LanBeacon`, simultaneous NAT `PunchProbe`
//! bursts, and `GangliaRelayFrame` forwarding.

use std::net::SocketAddr;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use skynet_agent::{AgentError, AgentNode};
use skynet_cell::HardwareTier;
use skynet_ledger::{NanoSkn, TxOutput, UtxoDagLedger};
use skynet_wire::{
    EpidermalFilter, GangliaRelayFrame, ImpulseBody, ImpulsePacket, LanBeacon, PeerContact,
    PheromoneHello, PheromoneHelloParams, PunchProbe, PunchRendezvous, RendezvousMessage, WireError,
};
use tokio::net::UdpSocket;

use crate::genesis_io::{load_genesis_file, GenesisIoError};

pub const LAN_BEACON_INTERVAL_MS: u64 = 500;
const STANDALONE_BOOTSTRAP_SKN: u128 = 100_000;

struct SocketAgentCtx<'a> {
    socket: &'a UdpSocket,
    agent: &'a mut AgentNode,
    now_sec: u64,
}

/// Bootstraps `UtxoDagLedger` from `--genesis <PATH>` when provided, or falls back to
/// standalone single-node allocation for backward-compatible local runs.
pub fn bootstrap_node_ledger(
    genesis_path: Option<&Path>,
    local_pubkey: [u8; 32],
) -> Result<(UtxoDagLedger, Option<[u8; 32]>), GenesisIoError> {
    if let Some(path) = genesis_path {
        let (cfg, hash) = load_genesis_file(path)?;
        let (ledger, _vault, _) = cfg.build_ledger_and_vault()?;
        return Ok((ledger, Some(hash)));
    }
    let init_alloc = NanoSkn::from_skn(STANDALONE_BOOTSTRAP_SKN)?;
    let (ledger, _) = UtxoDagLedger::from_genesis(vec![TxOutput::direct(local_pubkey, init_alloc)])?;
    Ok((ledger, None))
}

/// Stateful P2P wire framer, Seed rendezvous broker, and NAT hole-punch coordinator.
#[derive(Debug, Clone)]
pub struct RuntimeP2pDriver {
    pub genesis_hash: Option<[u8; 32]>,
    pub local_port: u16,
    pub lan_targets: Vec<SocketAddr>,
    hello_nonce: u64,
}

impl RuntimeP2pDriver {
    pub fn new(genesis_hash: Option<[u8; 32]>, local_port: u16, agent: &mut AgentNode) -> Self {
        if let Some(gh) = genesis_hash {
            agent.rendezvous.set_expected_genesis_hash(gh);
        }
        Self { genesis_hash, local_port, lan_targets: Vec::new(), hello_nonce: 0 }
    }

    pub fn add_lan_target(&mut self, addr: SocketAddr) {
        if !self.lan_targets.contains(&addr) {
            self.lan_targets.push(addr);
        }
    }

    pub fn encode_wire_packet(&self, pkt: &ImpulsePacket, now_sec: u64) -> Result<Vec<u8>, WireError> {
        match self.genesis_hash {
            Some(gh) => EpidermalFilter::encode_framed_packet(pkt, &gh, now_sec),
            None => pkt.encode_datagram(),
        }
    }

    pub fn unwrap_epidermal<'a>(&self, raw: &'a [u8], now_sec: u64) -> Result<&'a [u8], WireError> {
        match self.genesis_hash {
            Some(gh) => EpidermalFilter::verify_stack_header(raw, &gh, now_sec),
            None => Ok(raw),
        }
    }

    /// Sends signed `PheromoneHello` keepalive packets (`<= 300B`) to all `seed_peers`.
    pub async fn send_seed_keepalives(
        &mut self,
        socket: &UdpSocket,
        agent: &mut AgentNode,
        seed_peers: &[SocketAddr],
    ) -> Result<usize, WireError> {
        if seed_peers.is_empty() {
            return Ok(0);
        }
        let mut ctx = SocketAgentCtx { socket, agent, now_sec: unix_now_sec() };
        self.hello_nonce = self.hello_nonce.wrapping_add(1);
        let params = PheromoneHelloParams {
            internal_addr: SocketAddr::from(([127, 0, 0, 1], self.local_port)),
            node_tier: tier_to_u8(ctx.agent.profile.tier),
            timestamp_sec: ctx.now_sec,
            nonce: self.hello_nonce,
        };
        let hello = PheromoneHello::new_signed(&ctx.agent.signing_key, params)?;
        let wire = self.sign_and_frame(&mut ctx, 2, ImpulseBody::Rendezvous(RendezvousMessage::PheromoneHello(hello)))?;
        for &seed in seed_peers {
            let _ = ctx.socket.send_to(&wire, seed).await;
        }
        Ok(seed_peers.len())
    }

    /// Emits `< 1s` `LanBeacon` packets to configured local subnet targets.
    pub async fn broadcast_lan_beacon(&self, socket: &UdpSocket, agent: &mut AgentNode) -> Result<usize, WireError> {
        if self.lan_targets.is_empty() {
            return Ok(0);
        }
        let mut ctx = SocketAgentCtx { socket, agent, now_sec: unix_now_sec() };
        let beacon = LanBeacon {
            sender_pubkey: ctx.agent.pubkey,
            listen_port: self.local_port,
            genesis_hash: self.genesis_hash.unwrap_or([0u8; 32]),
        };
        let wire = self.sign_and_frame(&mut ctx, 1, ImpulseBody::Rendezvous(RendezvousMessage::LanBeacon(beacon)))?;
        for &target in &self.lan_targets {
            let _ = ctx.socket.send_to(&wire, target).await;
        }
        Ok(self.lan_targets.len())
    }

    /// Filters, decodes, routes, and answers a single inbound UDP datagram.
    pub async fn process_datagram(
        &mut self,
        socket: &UdpSocket,
        agent: &mut AgentNode,
        inbound: (&[u8], SocketAddr),
    ) -> Result<bool, AgentError> {
        let mut ctx = SocketAgentCtx { socket, agent, now_sec: unix_now_sec() };
        let Ok(inner) = self.unwrap_epidermal(inbound.0, ctx.now_sec) else { return Ok(false) };
        let Ok(pkt) = ImpulsePacket::decode_and_verify(inner) else { return Ok(false) };
        if pkt.sender_pubkey == ctx.agent.pubkey {
            return Ok(false);
        }
        if let ImpulseBody::Rendezvous(rmsg) = pkt.body {
            return self.handle_rendezvous_datagram(&mut ctx, rmsg, inbound.1).await;
        }
        if let Some(reply) = ctx.agent.handle_incoming_datagram_from(inner, Some(inbound.1))? {
            if let Ok(wire) = self.encode_wire_packet(&reply, ctx.now_sec) {
                let _ = ctx.socket.send_to(&wire, inbound.1).await;
            }
        }
        Ok(true)
    }

    async fn handle_rendezvous_datagram(&mut self, ctx: &mut SocketAgentCtx<'_>, msg: RendezvousMessage, src_addr: SocketAddr) -> Result<bool, AgentError> {
        match msg {
            RendezvousMessage::PheromoneHello(hello) => {
                let entry = ctx.agent.rendezvous.register_hello(&hello, src_addr, ctx.now_sec)?;
                record_contact(ctx.agent, entry.pubkey, entry.public_addr, 100);
                ctx.agent.udp_received += 1;
                self.broker_nat_pairings(ctx, hello.sender_pubkey).await?;
            }
            RendezvousMessage::PunchRendezvous(cmd) => {
                ctx.agent.udp_received += 1;
                self.fire_hole_punch_burst(ctx, &cmd).await?;
            }
            RendezvousMessage::PunchProbe(probe) => self.on_inbound_probe(ctx, &probe, src_addr).await?,
            RendezvousMessage::LanBeacon(beacon) => {
                let entry = ctx.agent.rendezvous.handle_lan_beacon(&beacon, src_addr.ip(), ctx.now_sec)?;
                record_contact(ctx.agent, entry.pubkey, entry.public_addr, 200);
                ctx.agent.udp_received += 1;
            }
            RendezvousMessage::GangliaRelayFrame(frame) => self.handle_relay_frame(ctx, frame, src_addr).await?,
        }
        Ok(true)
    }

    async fn broker_nat_pairings(&self, ctx: &mut SocketAgentCtx<'_>, newcomer: [u8; 32]) -> Result<(), AgentError> {
        let others: Vec<[u8; 32]> = ctx.agent.rendezvous.active_peers().into_iter().map(|p| p.pubkey).filter(|pk| *pk != newcomer).collect();
        for other_pk in others {
            let pair = ctx.agent.rendezvous.pair_nat_peers(&ctx.agent.signing_key, (other_pk, newcomer), ctx.now_sec)?;
            for (dest, cmd) in [pair.to_peer_a, pair.to_peer_b] {
                let wire = self.sign_and_frame(ctx, 2, ImpulseBody::Rendezvous(RendezvousMessage::PunchRendezvous(cmd)))?;
                let _ = ctx.socket.send_to(&wire, dest).await;
            }
        }
        Ok(())
    }

    async fn fire_hole_punch_burst(&self, ctx: &mut SocketAgentCtx<'_>, cmd: &PunchRendezvous) -> Result<(), AgentError> {
        let (target_addr, probes) = ctx.agent.rendezvous.build_punch_probe_burst(ctx.agent.pubkey, cmd, ctx.now_sec)?;
        for probe in probes {
            let wire = self.sign_and_frame(ctx, 2, ImpulseBody::Rendezvous(RendezvousMessage::PunchProbe(probe)))?;
            let _ = ctx.socket.send_to(&wire, target_addr).await;
        }
        Ok(())
    }

    async fn on_inbound_probe(&self, ctx: &mut SocketAgentCtx<'_>, probe: &PunchProbe, src_addr: SocketAddr) -> Result<(), AgentError> {
        let was_direct = ctx.agent.rendezvous.get_peer(&probe.sender_pubkey).map(|p| p.direct_reachable).unwrap_or(false);
        ctx.agent.udp_received += 1;
        let Some(entry) = ctx.agent.rendezvous.handle_punch_probe(probe, src_addr, ctx.now_sec) else { return Ok(()) };
        record_contact(ctx.agent, entry.pubkey, entry.public_addr, 150);
        if !was_direct {
            let ack = PunchProbe { sender_pubkey: ctx.agent.pubkey, rendezvous_token: probe.rendezvous_token };
            let wire = self.sign_and_frame(ctx, 2, ImpulseBody::Rendezvous(RendezvousMessage::PunchProbe(ack)))?;
            let _ = ctx.socket.send_to(&wire, src_addr).await;
        }
        Ok(())
    }

    async fn handle_relay_frame(&self, ctx: &mut SocketAgentCtx<'_>, frame: GangliaRelayFrame, src_addr: SocketAddr) -> Result<(), AgentError> {
        if frame.target_pubkey == ctx.agent.pubkey {
            if let Some(reply) = ctx.agent.handle_incoming_datagram_from(&frame.inner_datagram, Some(src_addr))? {
                let _ = ctx.socket.send_to(&self.encode_wire_packet(&reply, ctx.now_sec)?, src_addr).await;
            }
            return Ok(());
        }
        let route = ctx.agent.rendezvous.route_ganglia_relay(frame, ctx.now_sec)?;
        ctx.agent.udp_received += 1;
        let wire = self.sign_and_frame(ctx, 4, ImpulseBody::Rendezvous(RendezvousMessage::GangliaRelayFrame(route.frame)))?;
        let _ = ctx.socket.send_to(&wire, route.target_addr).await;
        Ok(())
    }

    fn sign_and_frame(&self, ctx: &mut SocketAgentCtx<'_>, hop_ttl: u8, body: ImpulseBody) -> Result<Vec<u8>, WireError> {
        ctx.agent.pulse_seq = ctx.agent.pulse_seq.wrapping_add(1);
        let pkt = ImpulsePacket::new_signed(&ctx.agent.signing_key, ctx.agent.cluster_prefix, ctx.agent.prefix_len, hop_ttl, ctx.agent.epoch, ctx.agent.pulse_seq, body)?;
        self.encode_wire_packet(&pkt, ctx.now_sec)
    }
}

fn record_contact(agent: &mut AgentNode, pubkey: [u8; 32], addr: SocketAddr, score: u16) {
    agent.observe_peer_contact(PeerContact { pubkey, addr, cluster_prefix: 0, prefix_len: 0, bonded_skn_whole: 1, synaptic_score: score });
}

fn tier_to_u8(tier: HardwareTier) -> u8 {
    match tier { HardwareTier::TinyPeripheral => 1, HardwareTier::StandardNode => 2, HardwareTier::ColossalOrganism => 3 }
}

pub fn unix_now_sec() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
