//! `rendezvous_wire`: P2P LAN beacon, Seed NAT hole-punch rendezvous (`<= 300B`),
//! and symmetric-NAT fallback `GangliaRelayFrame` (`<= 1200B`) wire messages.

use std::net::SocketAddr;

use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

use crate::packet::{
    sign_ed25519_digest, verify_ed25519_sig, WireError, MAX_UDP_DATAGRAM_BYTES,
};

/// Strict ceiling (`300 bytes`) for NAT hole-punch and rendezvous control payloads.
pub const MAX_RENDEZVOUS_CONTROL_BYTES: usize = 300;
/// Maximum inner datagram length inside `GangliaRelayFrame` so the outer packet stays `<= 1200B`.
pub const MAX_GANGLIA_RELAY_INNER_BYTES: usize = 948;

/// Input fields for constructing a signed `PheromoneHello` keepalive/registration.
#[derive(Debug, Clone, Copy)]
pub struct PheromoneHelloParams {
    pub internal_addr: SocketAddr,
    pub node_tier: u8,
    pub timestamp_sec: u64,
    pub nonce: u64,
}

/// Periodic (`25s`) worker/peer registration & NAT pinhole keepalive sent to a Seed node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PheromoneHello {
    pub sender_pubkey: [u8; 32],
    pub internal_addr: SocketAddr,
    pub node_tier: u8,
    pub timestamp_sec: u64,
    pub nonce: u64,
    pub signature: Vec<u8>,
}

impl PheromoneHello {
    pub fn new_signed(
        signing_key: &SigningKey,
        params: PheromoneHelloParams,
    ) -> Result<Self, WireError> {
        let mut msg = Self {
            sender_pubkey: signing_key.verifying_key().to_bytes(),
            internal_addr: params.internal_addr,
            node_tier: params.node_tier,
            timestamp_sec: params.timestamp_sec,
            nonce: params.nonce,
            signature: Vec::new(),
        };
        msg.signature = sign_ed25519_digest(signing_key, &msg.signing_digest()?);
        Ok(msg)
    }

    pub fn signing_digest(&self) -> Result<[u8; 32], WireError> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SKYNET_PHEROMONE_HELLO_V1");
        hasher.update(&self.sender_pubkey);
        hasher.update(&bincode::serialize(&self.internal_addr)?);
        hasher.update(&[self.node_tier]);
        hasher.update(&self.timestamp_sec.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        Ok(*hasher.finalize().as_bytes())
    }

    pub fn verify_signature(&self) -> Result<(), WireError> {
        verify_ed25519_sig(&self.sender_pubkey, &self.signing_digest()?, &self.signature)
    }
}

/// Input fields for constructing a Seed-signed `PunchRendezvous` pairing instruction.
#[derive(Debug, Clone, Copy)]
pub struct PunchRendezvousParams {
    pub target_peer_pubkey: [u8; 32],
    pub peer_public_addr: SocketAddr,
    pub rendezvous_token: u64,
    pub expiry_timestamp: u64,
}

/// Seed-brokered instruction (`<= 300B`) directing two NAT peers to fire simultaneous UDP probes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PunchRendezvous {
    pub target_peer_pubkey: [u8; 32],
    pub peer_public_addr: SocketAddr,
    pub rendezvous_token: u64,
    pub expiry_timestamp: u64,
    pub signature: Vec<u8>,
}

impl PunchRendezvous {
    pub fn new_signed(
        seed_signing_key: &SigningKey,
        params: PunchRendezvousParams,
    ) -> Result<Self, WireError> {
        let mut msg = Self {
            target_peer_pubkey: params.target_peer_pubkey,
            peer_public_addr: params.peer_public_addr,
            rendezvous_token: params.rendezvous_token,
            expiry_timestamp: params.expiry_timestamp,
            signature: Vec::new(),
        };
        msg.signature = sign_ed25519_digest(seed_signing_key, &msg.signing_digest()?);
        Ok(msg)
    }

    pub fn signing_digest(&self) -> Result<[u8; 32], WireError> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SKYNET_PUNCH_RENDEZVOUS_V1");
        hasher.update(&self.target_peer_pubkey);
        hasher.update(&bincode::serialize(&self.peer_public_addr)?);
        hasher.update(&self.rendezvous_token.to_le_bytes());
        hasher.update(&self.expiry_timestamp.to_le_bytes());
        Ok(*hasher.finalize().as_bytes())
    }

    pub fn verify_signature(&self, seed_pubkey: &[u8; 32]) -> Result<(), WireError> {
        verify_ed25519_sig(seed_pubkey, &self.signing_digest()?, &self.signature)
    }
}

/// Lightweight simultaneous UDP hole-punch burst probe sent directly to `peer_public_addr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PunchProbe {
    pub sender_pubkey: [u8; 32],
    pub rendezvous_token: u64,
}

/// Zero-configuration local subnet discovery beacon (`< 1s` LAN peer detection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanBeacon {
    pub sender_pubkey: [u8; 32],
    pub listen_port: u16,
    pub genesis_hash: [u8; 32],
}

/// Fallback relay envelope (`<= 1200B`) when symmetric NAT blocks direct hole punching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GangliaRelayFrame {
    pub target_pubkey: [u8; 32],
    pub sender_pubkey: [u8; 32],
    pub inner_datagram: Vec<u8>,
}

impl GangliaRelayFrame {
    pub fn new(
        target_pubkey: [u8; 32],
        sender_pubkey: [u8; 32],
        inner_datagram: Vec<u8>,
    ) -> Result<Self, WireError> {
        if inner_datagram.len() > MAX_GANGLIA_RELAY_INNER_BYTES {
            return Err(WireError::OversizedDatagram { actual: inner_datagram.len() });
        }
        Ok(Self { target_pubkey, sender_pubkey, inner_datagram })
    }
}

/// P2P discovery, NAT hole-punching, and relay messages carried in `ImpulseBody::Rendezvous`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RendezvousMessage {
    PheromoneHello(PheromoneHello),
    PunchRendezvous(PunchRendezvous),
    PunchProbe(PunchProbe),
    LanBeacon(LanBeacon),
    GangliaRelayFrame(GangliaRelayFrame),
}

impl RendezvousMessage {
    pub fn validate_mtu(&self) -> Result<(), WireError> {
        let len = bincode::serialized_size(self)? as usize;
        if let Self::GangliaRelayFrame(frame) = self {
            if frame.inner_datagram.len() > MAX_GANGLIA_RELAY_INNER_BYTES
                || len > MAX_UDP_DATAGRAM_BYTES
            {
                return Err(WireError::OversizedDatagram { actual: len });
            }
            return Ok(());
        }
        if len > MAX_RENDEZVOUS_CONTROL_BYTES {
            return Err(WireError::OversizedRendezvousControl {
                actual: len,
                max: MAX_RENDEZVOUS_CONTROL_BYTES,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::{ImpulseBody, ImpulsePacket};

    #[test]
    fn test_pheromone_and_punch_rendezvous_signatures_and_300b_budget() {
        let peer_key = SigningKey::from_bytes(&[11u8; 32]);
        let seed_key = SigningKey::from_bytes(&[22u8; 32]);
        let seed_pub = seed_key.verifying_key().to_bytes();

        let hello = PheromoneHello::new_signed(
            &peer_key,
            PheromoneHelloParams {
                internal_addr: "192.168.1.50:8888".parse().unwrap(),
                node_tier: 2,
                timestamp_sec: 1_700_000_000,
                nonce: 42,
            },
        )
        .unwrap();
        hello.verify_signature().unwrap();
        RendezvousMessage::PheromoneHello(hello).validate_mtu().unwrap();

        let punch = PunchRendezvous::new_signed(
            &seed_key,
            PunchRendezvousParams {
                target_peer_pubkey: [0xAB; 32],
                peer_public_addr: "203.0.113.9:49152".parse().unwrap(),
                rendezvous_token: 0xDEAD_BEEF,
                expiry_timestamp: 1_700_000_120,
            },
        )
        .unwrap();
        punch.verify_signature(&seed_pub).unwrap();
        assert!(punch.verify_signature(&[0xFF; 32]).is_err());

        let body = ImpulseBody::Rendezvous(RendezvousMessage::PunchRendezvous(punch));
        let pkt = ImpulsePacket::new_signed(&seed_key, 0, 0, 4, 1, 1, body).unwrap();
        assert!(pkt.encode_datagram().unwrap().len() <= MAX_RENDEZVOUS_CONTROL_BYTES);
    }

    #[test]
    fn test_lan_beacon_probe_and_ganglia_relay_mtu_bounds() {
        let key = SigningKey::from_bytes(&[33u8; 32]);
        let probe = RendezvousMessage::PunchProbe(PunchProbe { sender_pubkey: [1; 32], rendezvous_token: 7 });
        let beacon = RendezvousMessage::LanBeacon(LanBeacon { sender_pubkey: [1; 32], listen_port: 8888, genesis_hash: [2; 32] });
        probe.validate_mtu().unwrap();
        beacon.validate_mtu().unwrap();

        let frame = GangliaRelayFrame::new([1u8; 32], [2u8; 32], vec![0x7Fu8; 800]).unwrap();
        let body = ImpulseBody::Rendezvous(RendezvousMessage::GangliaRelayFrame(frame));
        let pkt = ImpulsePacket::new_signed(&key, 0, 0, 4, 1, 2, body).unwrap();
        assert!(pkt.encode_datagram().unwrap().len() <= MAX_UDP_DATAGRAM_BYTES);
        assert!(GangliaRelayFrame::new([1u8; 32], [2u8; 32], vec![0u8; 1100]).is_err());
    }
}
