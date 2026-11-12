//! `rendezvous_broker`: P2P Seed rendezvous registry, simultaneous NAT UDP hole-punch
//! pairing (`<= 300B`), `< 1s` `LanBeacon` handling, and `GangliaRelayFrame` routing.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use skynet_wire::{
    GangliaRelayFrame, LanBeacon, PheromoneHello, PunchProbe, PunchRendezvous,
    PunchRendezvousParams, RendezvousMessage, WireError,
};
use thiserror::Error;

pub const RENDEZVOUS_TTL_SEC: u64 = 120;
pub const HOLE_PUNCH_BURST_COUNT: usize = 4;
pub const SEED_KEEPALIVE_INTERVAL_SEC: u64 = 25;
const TOKEN_DOMAIN: &[u8] = b"SKYNET_RENDEZVOUS_TOKEN_V1";

#[derive(Debug, Error)]
pub enum RendezvousError {
    #[error("wire error: {0}")]
    Wire(#[from] WireError),
    #[error("requested peer is not registered or has expired")]
    PeerNotRegistered,
    #[error("cannot pair a peer with itself")]
    SelfPairingRejected,
    #[error("rendezvous instruction expired at {expiry_sec} (now {now_sec})")]
    ExpiredRendezvous { expiry_sec: u64, now_sec: u64 },
    #[error("LAN beacon genesis hash does not match local network")]
    InvalidLanGenesisHash,
    #[error("punch rendezvous signature does not match any trusted seed")]
    UntrustedSeedSignature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerRendezvousEntry {
    pub pubkey: [u8; 32],
    pub public_addr: SocketAddr,
    pub internal_addr: SocketAddr,
    pub node_tier: u8,
    pub last_seen_sec: u64,
    pub last_nonce: u64,
    pub direct_reachable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendezvousPairCommands {
    pub rendezvous_token: u64,
    pub expiry_timestamp: u64,
    pub to_peer_a: (SocketAddr, PunchRendezvous),
    pub to_peer_b: (SocketAddr, PunchRendezvous),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayRouteDecision {
    pub target_addr: SocketAddr,
    pub frame: GangliaRelayFrame,
}

/// Stateful Seed & Edge P2P broker managing NAT peer registrations, hole punching, and relays.
#[derive(Debug, Clone, Default)]
pub struct RendezvousRegistry {
    peers: HashMap<[u8; 32], PeerRendezvousEntry>,
    active_tokens: HashMap<u64, ([u8; 32], SocketAddr, u64)>,
    expected_genesis_hash: Option<[u8; 32]>,
    trusted_seed_pubkeys: Vec<[u8; 32]>,
    next_token_seq: u64,
}

impl RendezvousRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn with_genesis_hash(genesis_hash: [u8; 32]) -> Self {
        Self { expected_genesis_hash: Some(genesis_hash), ..Self::default() }
    }

    pub fn set_expected_genesis_hash(&mut self, genesis_hash: [u8; 32]) {
        self.expected_genesis_hash = Some(genesis_hash);
    }

    pub fn add_trusted_seed(&mut self, seed_pubkey: [u8; 32]) {
        if !self.trusted_seed_pubkeys.contains(&seed_pubkey) {
            self.trusted_seed_pubkeys.push(seed_pubkey);
        }
    }

    pub fn peer_count(&self) -> usize { self.peers.len() }
    pub fn get_peer(&self, pubkey: &[u8; 32]) -> Option<&PeerRendezvousEntry> { self.peers.get(pubkey) }
    pub fn active_peers(&self) -> Vec<PeerRendezvousEntry> { self.peers.values().cloned().collect() }

    pub fn evict_expired(&mut self, now_sec: u64) -> usize {
        let before = self.peers.len();
        self.peers.retain(|_, p| now_sec.saturating_sub(p.last_seen_sec) <= RENDEZVOUS_TTL_SEC);
        self.active_tokens.retain(|_, (_, _, exp)| now_sec <= *exp);
        before.saturating_sub(self.peers.len())
    }

    pub fn register_hello(
        &mut self,
        hello: &PheromoneHello,
        observed_public_addr: SocketAddr,
        now_sec: u64,
    ) -> Result<PeerRendezvousEntry, RendezvousError> {
        hello.verify_signature()?;
        self.evict_expired(now_sec);
        let direct = self.peers.get(&hello.sender_pubkey)
            .map(|p| p.direct_reachable && p.public_addr == observed_public_addr)
            .unwrap_or(false);
        let entry = PeerRendezvousEntry {
            pubkey: hello.sender_pubkey, public_addr: observed_public_addr, internal_addr: hello.internal_addr,
            node_tier: hello.node_tier, last_seen_sec: now_sec, last_nonce: hello.nonce, direct_reachable: direct,
        };
        self.peers.insert(hello.sender_pubkey, entry.clone());
        Ok(entry)
    }

    pub fn pair_nat_peers(
        &mut self,
        seed_signing_key: &SigningKey,
        peers: ([u8; 32], [u8; 32]),
        now_sec: u64,
    ) -> Result<RendezvousPairCommands, RendezvousError> {
        self.evict_expired(now_sec);
        if peers.0 == peers.1 {
            return Err(RendezvousError::SelfPairingRejected);
        }
        let a = self.peers.get(&peers.0).cloned().ok_or(RendezvousError::PeerNotRegistered)?;
        let b = self.peers.get(&peers.1).cloned().ok_or(RendezvousError::PeerNotRegistered)?;
        self.next_token_seq = self.next_token_seq.wrapping_add(1);
        let token = derive_token(&seed_signing_key.verifying_key().to_bytes(), peers, now_sec, self.next_token_seq);
        let expiry = now_sec.saturating_add(RENDEZVOUS_TTL_SEC);
        let cmd_a = sign_punch(seed_signing_key, PunchRendezvousParams { target_peer_pubkey: b.pubkey, peer_public_addr: b.public_addr, rendezvous_token: token, expiry_timestamp: expiry })?;
        let cmd_b = sign_punch(seed_signing_key, PunchRendezvousParams { target_peer_pubkey: a.pubkey, peer_public_addr: a.public_addr, rendezvous_token: token, expiry_timestamp: expiry })?;
        Ok(RendezvousPairCommands { rendezvous_token: token, expiry_timestamp: expiry, to_peer_a: (a.public_addr, cmd_a), to_peer_b: (b.public_addr, cmd_b) })
    }

    pub fn build_punch_probe_burst(
        &mut self,
        local_pubkey: [u8; 32],
        cmd: &PunchRendezvous,
        now_sec: u64,
    ) -> Result<(SocketAddr, Vec<PunchProbe>), RendezvousError> {
        if now_sec > cmd.expiry_timestamp {
            return Err(RendezvousError::ExpiredRendezvous { expiry_sec: cmd.expiry_timestamp, now_sec });
        }
        if !self.trusted_seed_pubkeys.is_empty()
            && !self.trusted_seed_pubkeys.iter().any(|pk| cmd.verify_signature(pk).is_ok())
        {
            return Err(RendezvousError::UntrustedSeedSignature);
        }
        self.active_tokens.insert(cmd.rendezvous_token, (cmd.target_peer_pubkey, cmd.peer_public_addr, cmd.expiry_timestamp));
        let probe = PunchProbe { sender_pubkey: local_pubkey, rendezvous_token: cmd.rendezvous_token };
        Ok((cmd.peer_public_addr, vec![probe; HOLE_PUNCH_BURST_COUNT]))
    }

    pub fn handle_punch_probe(&mut self, probe: &PunchProbe, src_addr: SocketAddr, now_sec: u64) -> Option<PeerRendezvousEntry> {
        self.evict_expired(now_sec);
        let valid_token = self.active_tokens.get(&probe.rendezvous_token)
            .map(|(pk, _, exp)| pk == &probe.sender_pubkey && now_sec <= *exp)
            .unwrap_or(false);
        if !valid_token && !self.peers.contains_key(&probe.sender_pubkey) {
            return None;
        }
        let tier = self.peers.get(&probe.sender_pubkey).map(|p| p.node_tier).unwrap_or(1);
        let entry = PeerRendezvousEntry {
            pubkey: probe.sender_pubkey, public_addr: src_addr, internal_addr: src_addr,
            node_tier: tier, last_seen_sec: now_sec, last_nonce: probe.rendezvous_token, direct_reachable: true,
        };
        self.peers.insert(probe.sender_pubkey, entry.clone());
        Some(entry)
    }

    pub fn handle_lan_beacon(&mut self, beacon: &LanBeacon, src_ip: IpAddr, now_sec: u64) -> Result<PeerRendezvousEntry, RendezvousError> {
        if let Some(expected) = self.expected_genesis_hash {
            if beacon.genesis_hash != expected {
                return Err(RendezvousError::InvalidLanGenesisHash);
            }
        }
        let addr = SocketAddr::new(src_ip, beacon.listen_port);
        let entry = PeerRendezvousEntry {
            pubkey: beacon.sender_pubkey, public_addr: addr, internal_addr: addr,
            node_tier: 1, last_seen_sec: now_sec, last_nonce: 0, direct_reachable: true,
        };
        self.peers.insert(beacon.sender_pubkey, entry.clone());
        Ok(entry)
    }

    pub fn route_ganglia_relay(&mut self, frame: GangliaRelayFrame, now_sec: u64) -> Result<RelayRouteDecision, RendezvousError> {
        RendezvousMessage::GangliaRelayFrame(frame.clone()).validate_mtu()?;
        self.evict_expired(now_sec);
        let target = self.peers.get(&frame.target_pubkey).ok_or(RendezvousError::PeerNotRegistered)?;
        Ok(RelayRouteDecision { target_addr: target.public_addr, frame })
    }
}

fn sign_punch(seed_key: &SigningKey, params: PunchRendezvousParams) -> Result<PunchRendezvous, RendezvousError> {
    let cmd = PunchRendezvous::new_signed(seed_key, params)?;
    RendezvousMessage::PunchRendezvous(cmd.clone()).validate_mtu()?;
    Ok(cmd)
}

fn derive_token(seed_pk: &[u8; 32], peers: ([u8; 32], [u8; 32]), now_sec: u64, seq: u64) -> u64 {
    let mut h = blake3::Hasher::new();
    h.update(TOKEN_DOMAIN);
    h.update(seed_pk);
    h.update(&peers.0);
    h.update(&peers.1);
    h.update(&now_sec.to_le_bytes());
    h.update(&seq.to_le_bytes());
    let s = *h.finalize().as_bytes();
    u64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]])
}

#[cfg(test)]
mod tests {
    use super::*;
    use skynet_wire::PheromoneHelloParams;

    #[test]
    fn nat_pairing_probe_burst_lan_beacon_relay_and_ttl_eviction() {
        let (seed_k, k_a, k_b) = (SigningKey::from_bytes(&[1; 32]), SigningKey::from_bytes(&[2; 32]), SigningKey::from_bytes(&[3; 32]));
        let (pk_a, pk_b, ghash) = (k_a.verifying_key().to_bytes(), k_b.verifying_key().to_bytes(), [0xAA; 32]);
        let (ext_a, ext_b): (SocketAddr, SocketAddr) = ("203.0.113.10:41000".parse().unwrap(), "198.51.100.20:52000".parse().unwrap());
        let mut seed_reg = RendezvousRegistry::with_genesis_hash(ghash);
        let h_a = PheromoneHello::new_signed(&k_a, PheromoneHelloParams { internal_addr: "192.168.1.2:8888".parse().unwrap(), node_tier: 1, timestamp_sec: 1_000, nonce: 1 }).unwrap();
        let h_b = PheromoneHello::new_signed(&k_b, PheromoneHelloParams { internal_addr: "10.0.0.5:8888".parse().unwrap(), node_tier: 2, timestamp_sec: 1_005, nonce: 1 }).unwrap();
        seed_reg.register_hello(&h_a, ext_a, 1_000).unwrap();
        seed_reg.register_hello(&h_b, ext_b, 1_005).unwrap();

        let pair = seed_reg.pair_nat_peers(&seed_k, (pk_a, pk_b), 1_010).unwrap();
        let mut reg_a = RendezvousRegistry::with_genesis_hash(ghash);
        reg_a.add_trusted_seed(seed_k.verifying_key().to_bytes());
        let (target_b, burst) = reg_a.build_punch_probe_burst(pk_a, &pair.to_peer_a.1, 1_012).unwrap();
        assert_eq!((target_b, burst.len()), (ext_b, HOLE_PUNCH_BURST_COUNT));
        let inbound_b = PunchProbe { sender_pubkey: pk_b, rendezvous_token: pair.rendezvous_token };
        assert!(reg_a.handle_punch_probe(&inbound_b, ext_b, 1_013).unwrap().direct_reachable);

        assert!(reg_a.handle_lan_beacon(&LanBeacon { sender_pubkey: [9; 32], listen_port: 8888, genesis_hash: ghash }, "192.168.1.9".parse().unwrap(), 1_015).is_ok());
        assert!(reg_a.handle_lan_beacon(&LanBeacon { sender_pubkey: [9; 32], listen_port: 8888, genesis_hash: [0; 32] }, "192.168.1.9".parse().unwrap(), 1_015).is_err());
        let relay = seed_reg.route_ganglia_relay(GangliaRelayFrame::new(pk_b, pk_a, vec![7u8; 64]).unwrap(), 1_020).unwrap();
        assert_eq!(relay.target_addr, ext_b);
        assert_eq!(seed_reg.evict_expired(1_000 + RENDEZVOUS_TTL_SEC + 1), 1);
    }
}
