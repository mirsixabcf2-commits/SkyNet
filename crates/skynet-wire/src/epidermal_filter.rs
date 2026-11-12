//! `epidermal_filter`: Zero-allocation `< 5µs` stack-based UDP ingress filter verifying
//! magic/version, `GENESIS_HASH` (`network_id`), and `±30s` anti-replay timestamp drift.

use serde::{Deserialize, Serialize};

use crate::packet::{
    ImpulsePacket, WireError, MAX_ALLOWED_HOP_TTL, MAX_UDP_DATAGRAM_BYTES, WIRE_MAGIC, WIRE_VERSION,
};

/// Fixed binary length of `EpidermalHeader` (`2 + 1 + 1 + 32 + 8 = 44 bytes`).
pub const EPIDERMAL_HEADER_BYTES: usize = 44;
/// Maximum allowed wall-clock drift (`±30 seconds`) for anti-replay protection.
pub const MAX_TIMESTAMP_DRIFT_SEC: u64 = 30;

/// Fixed 44-byte stack-allocated outer header binding a UDP datagram to `GENESIS_HASH`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpidermalHeader {
    pub magic: [u8; 2],
    pub version: u8,
    pub hop_ttl: u8,
    pub network_id: [u8; 32],
    pub timestamp_sec: u64,
}

impl EpidermalHeader {
    pub const fn new(network_id: [u8; 32], timestamp_sec: u64, hop_ttl: u8) -> Self {
        Self {
            magic: WIRE_MAGIC,
            version: WIRE_VERSION,
            hop_ttl,
            network_id,
            timestamp_sec,
        }
    }

    pub fn to_stack_bytes(&self) -> [u8; EPIDERMAL_HEADER_BYTES] {
        let mut buf = [0u8; EPIDERMAL_HEADER_BYTES];
        buf[0..2].copy_from_slice(&self.magic);
        buf[2] = self.version;
        buf[3] = self.hop_ttl;
        buf[4..36].copy_from_slice(&self.network_id);
        buf[36..44].copy_from_slice(&self.timestamp_sec.to_le_bytes());
        buf
    }

    pub fn from_stack_bytes(raw: &[u8]) -> Result<Self, WireError> {
        if raw.len() < EPIDERMAL_HEADER_BYTES {
            return Err(WireError::InvalidHeader);
        }
        let magic = [raw[0], raw[1]];
        let version = raw[2];
        let hop_ttl = raw[3];
        let mut network_id = [0u8; 32];
        network_id.copy_from_slice(&raw[4..36]);
        let mut ts_bytes = [0u8; 8];
        ts_bytes.copy_from_slice(&raw[36..44]);
        let timestamp_sec = u64::from_le_bytes(ts_bytes);
        Ok(Self {
            magic,
            version,
            hop_ttl,
            network_id,
            timestamp_sec,
        })
    }
}

/// Stateless epidermal boundary guard rejecting foreign-network or replayed datagrams
/// on stack slices in `< 5µs` with zero heap allocations on invalid inputs.
pub struct EpidermalFilter;

impl EpidermalFilter {
    pub fn verify_network_and_timestamp(
        network_id: &[u8; 32],
        expected_genesis_hash: &[u8; 32],
        packet_ts: u64,
        local_now_sec: u64,
    ) -> Result<(), WireError> {
        if network_id != expected_genesis_hash {
            return Err(WireError::NetworkIdMismatch);
        }
        let drift_sec = local_now_sec.abs_diff(packet_ts);
        if drift_sec > MAX_TIMESTAMP_DRIFT_SEC {
            return Err(WireError::TimestampDriftExceeded {
                drift_sec,
                max_sec: MAX_TIMESTAMP_DRIFT_SEC,
            });
        }
        Ok(())
    }

    pub fn parse_and_verify_header<'a>(
        raw: &'a [u8],
        expected_genesis_hash: &[u8; 32],
        local_now_sec: u64,
    ) -> Result<(EpidermalHeader, &'a [u8]), WireError> {
        if raw.len() > MAX_UDP_DATAGRAM_BYTES {
            return Err(WireError::OversizedDatagram { actual: raw.len() });
        }
        let hdr = EpidermalHeader::from_stack_bytes(raw)?;
        if hdr.magic != WIRE_MAGIC || hdr.version != WIRE_VERSION {
            return Err(WireError::InvalidHeader);
        }
        if hdr.hop_ttl == 0 || hdr.hop_ttl > MAX_ALLOWED_HOP_TTL {
            return Err(WireError::InvalidHopTtl {
                actual: hdr.hop_ttl,
                max: MAX_ALLOWED_HOP_TTL,
            });
        }
        Self::verify_network_and_timestamp(
            &hdr.network_id,
            expected_genesis_hash,
            hdr.timestamp_sec,
            local_now_sec,
        )?;
        Ok((hdr, &raw[EPIDERMAL_HEADER_BYTES..]))
    }

    pub fn verify_stack_header<'a>(
        raw: &'a [u8],
        expected_genesis_hash: &[u8; 32],
        local_now_sec: u64,
    ) -> Result<&'a [u8], WireError> {
        let (_, inner_slice) =
            Self::parse_and_verify_header(raw, expected_genesis_hash, local_now_sec)?;
        Ok(inner_slice)
    }

    pub fn encode_framed(
        header: &EpidermalHeader,
        inner_datagram: &[u8],
    ) -> Result<Vec<u8>, WireError> {
        if header.magic != WIRE_MAGIC || header.version != WIRE_VERSION {
            return Err(WireError::InvalidHeader);
        }
        if header.hop_ttl == 0 || header.hop_ttl > MAX_ALLOWED_HOP_TTL {
            return Err(WireError::InvalidHopTtl {
                actual: header.hop_ttl,
                max: MAX_ALLOWED_HOP_TTL,
            });
        }
        let total_len = EPIDERMAL_HEADER_BYTES + inner_datagram.len();
        if total_len > MAX_UDP_DATAGRAM_BYTES {
            return Err(WireError::OversizedDatagram { actual: total_len });
        }
        let mut framed = Vec::with_capacity(total_len);
        framed.extend_from_slice(&header.to_stack_bytes());
        framed.extend_from_slice(inner_datagram);
        Ok(framed)
    }

    pub fn encode_framed_packet(
        pkt: &ImpulsePacket,
        network_id: &[u8; 32],
        timestamp_sec: u64,
    ) -> Result<Vec<u8>, WireError> {
        let inner = pkt.encode_datagram()?;
        let hdr = EpidermalHeader::new(*network_id, timestamp_sec, pkt.hop_ttl);
        Self::encode_framed(&hdr, &inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::ImpulseBody;
    use ed25519_dalek::SigningKey;
    use std::time::Instant;

    fn sample_inner_packet() -> ImpulsePacket {
        let key = SigningKey::from_bytes(&[3u8; 32]);
        ImpulsePacket::new_signed(
            &key,
            0,
            0,
            4,
            1,
            1,
            ImpulseBody::Heartbeat {
                tier_code: 1,
                total_cells: 8,
                stem_cells: 2,
                on_chain_nano_skn: 500,
                elite_dna_hash: [0xAB; 32],
            },
        )
        .unwrap()
    }

    #[test]
    fn test_epidermal_header_44_byte_roundtrip_and_packet_compat() {
        let genesis = [0x42u8; 32];
        let pkt = sample_inner_packet();
        let framed = EpidermalFilter::encode_framed_packet(&pkt, &genesis, 1_700_000_000).unwrap();
        assert_eq!(framed.len(), EPIDERMAL_HEADER_BYTES + pkt.encode_datagram().unwrap().len());
        assert!(framed.len() <= MAX_UDP_DATAGRAM_BYTES);

        let inner = EpidermalFilter::verify_stack_header(&framed, &genesis, 1_700_000_030).unwrap();
        let decoded = ImpulsePacket::decode_and_verify(inner).unwrap();
        assert_eq!(decoded.sender_pubkey, pkt.sender_pubkey);
    }

    #[test]
    fn test_epidermal_rejects_wrong_genesis_and_drift_under_5_micros() {
        let genesis = [0x11u8; 32];
        let wrong_genesis = [0x22u8; 32];
        let hdr = EpidermalHeader::new(wrong_genesis, 1_000, 4);
        let bad_net = EpidermalFilter::encode_framed(&hdr, &[1, 2, 3, 4]).unwrap();

        let err_net = EpidermalFilter::verify_stack_header(&bad_net, &genesis, 1_000).unwrap_err();
        assert!(matches!(err_net, WireError::NetworkIdMismatch));

        let hdr_ok = EpidermalHeader::new(genesis, 1_000, 4);
        let stale = EpidermalFilter::encode_framed(&hdr_ok, &[1, 2, 3, 4]).unwrap();
        let err_ts = EpidermalFilter::verify_stack_header(&stale, &genesis, 1_031).unwrap_err();
        assert!(matches!(err_ts, WireError::TimestampDriftExceeded { drift_sec: 31, .. }));

        let start = Instant::now();
        for _ in 0..1_000 {
            let _ = std::hint::black_box(EpidermalFilter::verify_stack_header(
                std::hint::black_box(&bad_net),
                &genesis,
                1_000,
            ));
        }
        let avg_nanos = start.elapsed().as_nanos() / 1_000;
        assert!(avg_nanos < 5_000, "Expected < 5000ns, measured {avg_nanos}ns");
    }
}
