//! `packet`: Authenticated `<= 1200-byte` Synaptic UDP `ImpulsePacket`, `ImpulseBody`,
//! and `WireError` definitions.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use skynet_ledger::{SknTransaction, SynapticVoucher, TxOutput, UtreexoProof, UtxoId};
use thiserror::Error;

use crate::compute_wire::ComputeMessage;
use crate::fountain::FountainDroplet;
use crate::micro_train_wire::MicroTrainWireMessage;
use crate::rendezvous_wire::RendezvousMessage;

/// Strict IPv4/IPv6 fragmentation-free UDP datagram ceiling (`1200 bytes`).
pub const MAX_UDP_DATAGRAM_BYTES: usize = 1200;
pub const DEFAULT_PORT: u16 = 8888;
pub const WIRE_MAGIC: [u8; 2] = *b"SK";
pub const WIRE_VERSION: u8 = 1;
pub const MAX_ALLOWED_HOP_TTL: u8 = 16;

#[derive(Debug, Error)]
pub enum WireError {
    #[error("UDP impulse exceeds 1200-byte MTU ceiling ({actual} bytes)")]
    OversizedDatagram { actual: usize },
    #[error("rendezvous control message exceeds {max}-byte ceiling ({actual} bytes)")]
    OversizedRendezvousControl { actual: usize, max: usize },
    #[error("micro-training shard exceeds {max}-byte ceiling ({actual} bytes)")]
    OversizedShard { actual: usize, max: usize },
    #[error("invalid magic bytes or unsupported wire version")]
    InvalidHeader,
    #[error("invalid hop TTL ({actual}): must be between 1 and {max}")]
    InvalidHopTtl { actual: u8, max: u8 },
    #[error("epidermal filter rejected packet: network_id mismatch")]
    NetworkIdMismatch,
    #[error("epidermal filter rejected packet: timestamp drift {drift_sec}s exceeds ±{max_sec}s")]
    TimestampDriftExceeded { drift_sec: u64, max_sec: u64 },
    #[error("bincode serialization error: {0}")]
    Codec(#[from] bincode::Error),
    #[error("invalid Ed25519 packet signature")]
    BadSignature,
    #[error("fountain object exceeds maximum supported blocks")]
    FountainObjectTooLarge,
    #[error("fountain BLAKE3 digest mismatch after reconstruction")]
    FountainChecksumMismatch,
}

pub(crate) fn sign_ed25519_digest(signing_key: &SigningKey, digest: &[u8; 32]) -> Vec<u8> {
    let sig: Signature = signing_key.sign(digest);
    sig.to_bytes().to_vec()
}

pub(crate) fn verify_ed25519_sig(
    pubkey: &[u8; 32],
    digest: &[u8; 32],
    sig_slice: &[u8],
) -> Result<(), WireError> {
    let vk = VerifyingKey::from_bytes(pubkey).map_err(|_| WireError::BadSignature)?;
    let sig_arr: [u8; 64] = sig_slice.try_into().map_err(|_| WireError::BadSignature)?;
    vk.verify(digest, &Signature::from_bytes(&sig_arr))
        .map_err(|_| WireError::BadSignature)
}

/// Payload variants carried inside a single `<= 1200B` UDP `ImpulsePacket`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpulseBody {
    Heartbeat {
        tier_code: u8,
        total_cells: u32,
        stem_cells: u32,
        on_chain_nano_skn: u128,
        elite_dna_hash: [u8; 32],
    },
    GossipTx(SknTransaction),
    WitnessedTx {
        tx: SknTransaction,
        spent_outputs: Vec<TxOutput>,
        proofs: Vec<UtreexoProof>,
    },
    SynapticMicroPay { voucher: SynapticVoucher, stimulus: [f32; 8] },
    SnowballPull { query_id: u64, contested_utxos: Vec<UtxoId> },
    SnowballReply { query_id: u64, votes: Vec<(UtxoId, [u8; 32])> },
    ClusterMitosis { parent_prefix: u32, parent_prefix_len: u8, epoch: u64 },
    FountainDroplet(FountainDroplet),
    Compute(ComputeMessage),
    Rendezvous(RendezvousMessage),
    MicroTrain(MicroTrainWireMessage),
}

/// Routing and sequence header options for constructing a signed `ImpulsePacket`.
#[derive(Debug, Clone, Copy)]
pub struct ImpulseSignOptions {
    pub cluster_prefix: u32,
    pub prefix_len: u8,
    pub hop_ttl: u8,
    pub epoch: u64,
    pub pulse_seq: u32,
}

/// Authenticated connectionless UDP packet (`<= 1200 bytes` on wire).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpulsePacket {
    pub magic: [u8; 2],
    pub version: u8,
    pub cluster_prefix: u32,
    pub prefix_len: u8,
    pub hop_ttl: u8,
    pub epoch: u64,
    pub pulse_seq: u32,
    pub sender_pubkey: [u8; 32],
    pub body: ImpulseBody,
    pub signature: Vec<u8>,
}

impl ImpulsePacket {
    pub fn new_with_options(
        signing_key: &SigningKey,
        opts: ImpulseSignOptions,
        body: ImpulseBody,
    ) -> Result<Self, WireError> {
        if let ImpulseBody::Rendezvous(ref msg) = body {
            msg.validate_mtu()?;
        }
        let mut pkt = Self {
            magic: WIRE_MAGIC,
            version: WIRE_VERSION,
            cluster_prefix: opts.cluster_prefix,
            prefix_len: opts.prefix_len,
            hop_ttl: opts.hop_ttl,
            epoch: opts.epoch,
            pulse_seq: opts.pulse_seq,
            sender_pubkey: signing_key.verifying_key().to_bytes(),
            body,
            signature: Vec::new(),
        };
        pkt.signature = sign_ed25519_digest(signing_key, &pkt.signing_digest()?);
        let encoded_len = bincode::serialized_size(&pkt)? as usize;
        if encoded_len > MAX_UDP_DATAGRAM_BYTES {
            return Err(WireError::OversizedDatagram { actual: encoded_len });
        }
        Ok(pkt)
    }

    pub fn new_signed(
        signing_key: &SigningKey,
        cluster_prefix: u32,
        prefix_len: u8,
        hop_ttl: u8,
        epoch: u64,
        pulse_seq: u32,
        body: ImpulseBody,
    ) -> Result<Self, WireError> {
        let opts = ImpulseSignOptions { cluster_prefix, prefix_len, hop_ttl, epoch, pulse_seq };
        Self::new_with_options(signing_key, opts, body)
    }

    pub fn signing_digest(&self) -> Result<[u8; 32], WireError> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"SKYNET_UDP_IMPULSE_V1");
        hasher.update(&self.magic);
        hasher.update(&[self.version]);
        hasher.update(&self.cluster_prefix.to_le_bytes());
        hasher.update(&[self.prefix_len, self.hop_ttl]);
        hasher.update(&self.epoch.to_le_bytes());
        hasher.update(&self.pulse_seq.to_le_bytes());
        hasher.update(&self.sender_pubkey);
        hasher.update(&bincode::serialize(&self.body)?);
        Ok(*hasher.finalize().as_bytes())
    }

    pub fn encode_datagram(&self) -> Result<Vec<u8>, WireError> {
        let bytes = bincode::serialize(self)?;
        if bytes.len() > MAX_UDP_DATAGRAM_BYTES {
            return Err(WireError::OversizedDatagram { actual: bytes.len() });
        }
        Ok(bytes)
    }

    pub fn decode_and_verify(raw: &[u8]) -> Result<Self, WireError> {
        if raw.len() > MAX_UDP_DATAGRAM_BYTES {
            return Err(WireError::OversizedDatagram { actual: raw.len() });
        }
        let pkt: Self = bincode::deserialize(raw)?;
        if pkt.magic != WIRE_MAGIC || pkt.version != WIRE_VERSION {
            return Err(WireError::InvalidHeader);
        }
        if pkt.hop_ttl == 0 || pkt.hop_ttl > MAX_ALLOWED_HOP_TTL {
            return Err(WireError::InvalidHopTtl { actual: pkt.hop_ttl, max: MAX_ALLOWED_HOP_TTL });
        }
        if let ImpulseBody::Rendezvous(ref msg) = pkt.body {
            msg.validate_mtu()?;
        }
        verify_ed25519_sig(&pkt.sender_pubkey, &pkt.signing_digest()?, &pkt.signature)?;
        Ok(pkt)
    }
}
