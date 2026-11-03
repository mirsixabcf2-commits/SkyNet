//! `skynet-wire`: Loss-tolerant `<= 1200-byte` Synaptic UDP wire protocol,
//! `Ed25519` datagram authentication, and ACK-less GF(2) Fountain Erasure Droplets.

use std::net::SocketAddr;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use skynet_ledger::{SknTransaction, SynapticVoucher, TxOutput, UtreexoProof, UtxoId};
use thiserror::Error;

/// Strict IPv4/IPv6 fragmentation-free UDP datagram ceiling (`1200 bytes`).
pub const MAX_UDP_DATAGRAM_BYTES: usize = 1200;
pub const WIRE_MAGIC: [u8; 2] = *b"SK";
pub const WIRE_VERSION: u8 = 1;
pub const FOUNTAIN_BLOCK_BYTES: usize = 512;

#[derive(Debug, Error)]
pub enum WireError {
    #[error("UDP impulse exceeds 1200-byte MTU ceiling ({actual} bytes)")]
    OversizedDatagram { actual: usize },
    #[error("invalid magic bytes or unsupported wire version")]
    InvalidHeader,
    #[error("bincode serialization error: {0}")]
    Codec(#[from] bincode::Error),
    #[error("invalid Ed25519 packet signature")]
    BadSignature,
    #[error("fountain object exceeds maximum supported blocks")]
    FountainObjectTooLarge,
    #[error("fountain BLAKE3 digest mismatch after reconstruction")]
    FountainChecksumMismatch,
}

/// Payload variants carried inside a single `<= 1200B` UDP `ImpulsePacket`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpulseBody {
    /// Periodic biological heartbeat announcing Agent scale, health, and stake.
    Heartbeat {
        tier_code: u8,
        total_cells: u32,
        stem_cells: u32,
        on_chain_nano_skn: u128,
        elite_dna_hash: [u8; 32],
    },
    /// Unsolicited gossip of a signed SKN UTXO-DAG transaction.
    GossipTx(SknTransaction),
    /// Statelessly verifiable transaction carrying compact `Utreexo` Merkle witnesses (`<= 1200B`).
    WitnessedTx {
        tx: SknTransaction,
        spent_outputs: Vec<TxOutput>,
        proofs: Vec<UtreexoProof>,
    },
    /// Loss-tolerant cumulative `nanoSKN` micro-payment voucher + AI stimulus impulse.
    SynapticMicroPay {
        voucher: SynapticVoucher,
        stimulus: [f32; 8],
    },
    /// Loss-tolerant Snowball consensus poll querying contested UTXOs.
    SnowballPull {
        query_id: u64,
        contested_utxos: Vec<UtxoId>,
    },
    /// Compact Snowball vote reply (`(contested_utxo, preferred_tx_hash)`).
    SnowballReply {
        query_id: u64,
        votes: Vec<(UtxoId, [u8; 32])>,
    },
    /// Announces dynamic cluster prefix fission (`0bP -> 0bP0, 0bP1`).
    ClusterMitosis {
        parent_prefix: u32,
        parent_prefix_len: u8,
        epoch: u64,
    },
    /// Stateless Fountain erasure-coded droplet for syncing large `CellDna` or state snapshots.
    FountainDroplet(FountainDroplet),
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
    pub fn new_signed(
        signing_key: &SigningKey,
        cluster_prefix: u32,
        prefix_len: u8,
        hop_ttl: u8,
        epoch: u64,
        pulse_seq: u32,
        body: ImpulseBody,
    ) -> Result<Self, WireError> {
        let sender_pubkey = signing_key.verifying_key().to_bytes();
        let mut pkt = Self {
            magic: WIRE_MAGIC,
            version: WIRE_VERSION,
            cluster_prefix,
            prefix_len,
            hop_ttl,
            epoch,
            pulse_seq,
            sender_pubkey,
            body,
            signature: Vec::new(),
        };
        let digest = pkt.signing_digest()?;
        let sig: Signature = signing_key.sign(&digest);
        pkt.signature = sig.to_bytes().to_vec();

        // Enforce strict 1200-byte MTU discipline at creation time
        let encoded_len = bincode::serialized_size(&pkt)? as usize;
        if encoded_len > MAX_UDP_DATAGRAM_BYTES {
            return Err(WireError::OversizedDatagram {
                actual: encoded_len,
            });
        }
        Ok(pkt)
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
        let body_bytes = bincode::serialize(&self.body)?;
        hasher.update(&body_bytes);
        Ok(*hasher.finalize().as_bytes())
    }

    pub fn encode_datagram(&self) -> Result<Vec<u8>, WireError> {
        let bytes = bincode::serialize(self)?;
        if bytes.len() > MAX_UDP_DATAGRAM_BYTES {
            return Err(WireError::OversizedDatagram {
                actual: bytes.len(),
            });
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
        let vk = VerifyingKey::from_bytes(&pkt.sender_pubkey)
            .map_err(|_| WireError::BadSignature)?;
        let sig_arr: [u8; 64] = pkt
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| WireError::BadSignature)?;
        let sig = Signature::from_bytes(&sig_arr);
        let digest = pkt.signing_digest()?;
        vk.verify(&digest, &sig)
            .map_err(|_| WireError::BadSignature)?;
        Ok(pkt)
    }
}

/// Single erasure-coded droplet (`~560 bytes`) for ACK-less transfer over lossy UDP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FountainDroplet {
    pub object_hash: [u8; 32],
    pub original_len: u32,
    pub num_blocks: u16,
    pub droplet_seq: u32,
    pub xor_chunk: Vec<u8>,
}

/// Deterministic GF(2) Fountain Encoder producing an endless stream of independent UDP droplets.
pub struct FountainEncoder {
    object_hash: [u8; 32],
    original_len: u32,
    blocks: Vec<[u8; FOUNTAIN_BLOCK_BYTES]>,
    next_seq: u32,
}

impl FountainEncoder {
    pub fn new(payload: &[u8]) -> Result<Self, WireError> {
        let object_hash = *blake3::hash(payload).as_bytes();
        let num_blocks = payload.len().div_ceil(FOUNTAIN_BLOCK_BYTES).max(1);
        if num_blocks > 512 {
            return Err(WireError::FountainObjectTooLarge);
        }

        let mut blocks = vec![[0u8; FOUNTAIN_BLOCK_BYTES]; num_blocks];
        for (i, chunk) in payload.chunks(FOUNTAIN_BLOCK_BYTES).enumerate() {
            blocks[i][..chunk.len()].copy_from_slice(chunk);
        }

        Ok(Self {
            object_hash,
            original_len: payload.len() as u32,
            blocks,
            next_seq: 0,
        })
    }

    pub fn num_blocks(&self) -> u16 {
        self.blocks.len() as u16
    }

    pub fn next_droplet(&mut self) -> FountainDroplet {
        let seq = self.next_seq;
        self.next_seq = self.next_seq.wrapping_add(1);
        let k = self.blocks.len();
        let coeffs = droplet_coefficients(&self.object_hash, k, seq);

        let mut xor_chunk = vec![0u8; FOUNTAIN_BLOCK_BYTES];
        for (idx, &selected) in coeffs.iter().enumerate() {
            if selected {
                for (dst, src) in xor_chunk.iter_mut().zip(self.blocks[idx].iter()) {
                    *dst ^= *src;
                }
            }
        }

        FountainDroplet {
            object_hash: self.object_hash,
            original_len: self.original_len,
            num_blocks: k as u16,
            droplet_seq: seq,
            xor_chunk,
        }
    }
}

/// On-the-fly GF(2) Gauss-Jordan Fountain Decoder reconstructing multi-KB objects from
/// any arbitrary full-rank subset of surviving UDP droplets.
pub struct FountainDecoder {
    object_hash: [u8; 32],
    original_len: usize,
    k: usize,
    pivots: Vec<Option<(Vec<bool>, Vec<u8>)>>,
    rank: usize,
}

impl FountainDecoder {
    pub fn from_first_droplet(d: &FountainDroplet) -> Self {
        let k = d.num_blocks as usize;
        Self {
            object_hash: d.object_hash,
            original_len: d.original_len as usize,
            k,
            pivots: vec![None; k],
            rank: 0,
        }
    }

    /// Absorbs one UDP droplet; returns `Ok(Some(reconstructed_bytes))` once full rank is reached.
    pub fn absorb(&mut self, droplet: FountainDroplet) -> Result<Option<Vec<u8>>, WireError> {
        if droplet.object_hash != self.object_hash
            || droplet.num_blocks as usize != self.k
            || droplet.xor_chunk.len() != FOUNTAIN_BLOCK_BYTES
        {
            return Ok(None);
        }
        if self.rank == self.k {
            return self.extract_verified();
        }

        let mut row_bits = droplet_coefficients(&self.object_hash, self.k, droplet.droplet_seq);
        let mut row_data = droplet.xor_chunk;

        for col in 0..self.k {
            if !row_bits[col] {
                continue;
            }
            if let Some((piv_bits, piv_data)) = &self.pivots[col] {
                for b in col..self.k {
                    row_bits[b] ^= piv_bits[b];
                }
                for (dst, src) in row_data.iter_mut().zip(piv_data.iter()) {
                    *dst ^= *src;
                }
            } else {
                // Eliminate `col` from any higher pivots below `col` first
                for below in (col + 1)..self.k {
                    if row_bits[below] {
                        if let Some((piv_bits, piv_data)) = &self.pivots[below] {
                            for b in below..self.k {
                                row_bits[b] ^= piv_bits[b];
                            }
                            for (dst, src) in row_data.iter_mut().zip(piv_data.iter()) {
                                *dst ^= *src;
                            }
                        }
                    }
                }
                // Back-substitute into earlier pivots so matrix stays in reduced echelon form
                for above in 0..col {
                    if let Some((piv_bits, piv_data)) = &mut self.pivots[above] {
                        if piv_bits[col] {
                            for b in col..self.k {
                                piv_bits[b] ^= row_bits[b];
                            }
                            for (dst, src) in piv_data.iter_mut().zip(row_data.iter()) {
                                *dst ^= *src;
                            }
                        }
                    }
                }
                self.pivots[col] = Some((row_bits, row_data));
                self.rank += 1;
                break;
            }
        }

        if self.rank == self.k {
            self.extract_verified()
        } else {
            Ok(None)
        }
    }

    fn extract_verified(&self) -> Result<Option<Vec<u8>>, WireError> {
        let mut out = Vec::with_capacity(self.k * FOUNTAIN_BLOCK_BYTES);
        for piv in &self.pivots {
            let (_, block) = piv.as_ref().expect("rank == k guarantees all pivots set");
            out.extend_from_slice(block);
        }
        out.truncate(self.original_len);
        if blake3::hash(&out).as_bytes() != &self.object_hash {
            return Err(WireError::FountainChecksumMismatch);
        }
        Ok(Some(out))
    }
}

fn droplet_coefficients(object_hash: &[u8; 32], k: usize, seq: u32) -> Vec<bool> {
    let mut seed = [0u8; 32];
    seed.copy_from_slice(object_hash);
    let seq_bytes = seq.to_le_bytes();
    for i in 0..4 {
        seed[i] ^= seq_bytes[i];
    }
    let mut rng = ChaCha8Rng::from_seed(seed);
    let mut bits = vec![false; k];
    // Sparse singleton 20% of the time, dense random GF(2) vector 80% of the time
    if rng.gen_bool(0.20) {
        bits[rng.gen_range(0..k)] = true;
    } else {
        let mut any = false;
        for b in &mut bits {
            let v = rng.gen_bool(0.5);
            *b = v;
            any |= v;
        }
        if !any {
            bits[rng.gen_range(0..k)] = true;
        }
    }
    bits
}

/// Compact peer contact record (`~68 bytes`) piggybacked on UDP gossip for trackerless PEX.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerContact {
    pub pubkey: [u8; 32],
    pub addr: SocketAddr,
    pub cluster_prefix: u32,
    pub prefix_len: u8,
    pub bonded_skn_whole: u64,
    pub synaptic_score: u16,
}

/// Bounded **`O(log2 N)` Synaptic Peer Routing Table** combining:
/// 1. **Intra-Cluster Active View (`HyParView`)**: Fixed cap of same-cluster peers for Snowball UDP polls.
/// 2. **Logarithmic Axon Buckets (`Kademlia XOR`)**: Fixed `k` slots per XOR distance bit (`0..64`),
///    reaching 1 Billion global nodes in `<= 8` hops using only **`~5–14 KB` of RAM**!
/// 3. **Hebbian Synaptic Pruning**: Evicts silent/zero-stake Sybil IPs while retaining responsive staked relays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapticPeerTable {
    local_node_hash: [u8; 32],
    pub local_prefix: u32,
    pub local_prefix_len: u8,
    pub max_intra_cluster: usize,
    pub max_per_xor_bucket: usize,
    pub intra_cluster_peers: Vec<PeerContact>,
    pub axon_xor_buckets: Vec<Vec<PeerContact>>,
    pub total_observed_ips_ever: u64,
    pub total_hebbian_evictions: u64,
}

impl SynapticPeerTable {
    pub fn new(
        local_pubkey: &[u8; 32],
        local_prefix: u32,
        local_prefix_len: u8,
        max_intra_cluster: usize,
        max_per_xor_bucket: usize,
    ) -> Self {
        let local_node_hash = *blake3::hash(local_pubkey).as_bytes();
        Self {
            local_node_hash,
            local_prefix,
            local_prefix_len,
            max_intra_cluster: max_intra_cluster.max(4),
            max_per_xor_bucket: max_per_xor_bucket.max(1),
            intra_cluster_peers: Vec::new(),
            axon_xor_buckets: vec![Vec::new(); 64],
            total_observed_ips_ever: 0,
            total_hebbian_evictions: 0,
        }
    }

    pub fn xor_bucket_index(&self, remote_pubkey: &[u8; 32]) -> usize {
        let remote_hash = blake3::hash(remote_pubkey);
        let rh = remote_hash.as_bytes();
        let mut leading = 0usize;
        for i in 0..8 {
            let x = self.local_node_hash[i] ^ rh[i];
            if x == 0 {
                leading += 8;
            } else {
                leading += x.leading_zeros() as usize;
                break;
            }
        }
        leading.min(63)
    }

    /// Inserts or refreshes a peer IP with strict $O(\log N)$ bounded memory!
    pub fn observe_peer(&mut self, mut contact: PeerContact) {
        self.total_observed_ips_ever = self.total_observed_ips_ever.saturating_add(1);
        let same_cluster = contact.prefix_len == self.local_prefix_len
            && contact.cluster_prefix == self.local_prefix;

        // Boost synaptic weight logarithmically by bonded SKN stake
        let stake_bonus = ((contact.bonded_skn_whole as f64 + 1.0).ln() * 25.0) as u16;
        contact.synaptic_score = contact.synaptic_score.saturating_add(100 + stake_bonus);

        if same_cluster {
            Self::insert_bounded(
                &mut self.intra_cluster_peers,
                self.max_intra_cluster,
                contact,
                &mut self.total_hebbian_evictions,
            );
        } else {
            let b_idx = self.xor_bucket_index(&contact.pubkey);
            Self::insert_bounded(
                &mut self.axon_xor_buckets[b_idx],
                self.max_per_xor_bucket,
                contact,
                &mut self.total_hebbian_evictions,
            );
        }
    }

    fn insert_bounded(
        slot_vec: &mut Vec<PeerContact>,
        cap: usize,
        incoming: PeerContact,
        evictions: &mut u64,
    ) {
        if let Some(existing) = slot_vec.iter_mut().find(|p| p.pubkey == incoming.pubkey) {
            existing.addr = incoming.addr;
            existing.synaptic_score = existing.synaptic_score.saturating_add(50).min(60_000);
            existing.bonded_skn_whole = incoming.bonded_skn_whole;
            return;
        }
        if slot_vec.len() < cap {
            slot_vec.push(incoming);
            return;
        }
        // Bucket full -> Hebbian competition: replace weakest peer ONLY if incoming is stronger
        if let Some((weak_idx, weak_peer)) = slot_vec
            .iter()
            .enumerate()
            .min_by_key(|(_, p)| p.synaptic_score)
        {
            if incoming.synaptic_score > weak_peer.synaptic_score {
                slot_vec[weak_idx] = incoming;
                *evictions = evictions.saturating_add(1);
            } else {
                // Decay incumbent slightly so dead/silent incumbents eventually yield
                slot_vec[weak_idx].synaptic_score =
                    slot_vec[weak_idx].synaptic_score.saturating_sub(15);
            }
        }
    }

    /// Decays all synaptic weights periodically ("forgetting" dead/unresponsive IPs).
    pub fn decay_synapses(&mut self) {
        for p in &mut self.intra_cluster_peers {
            p.synaptic_score = (p.synaptic_score * 9) / 10;
        }
        for bucket in &mut self.axon_xor_buckets {
            for p in bucket {
                p.synaptic_score = (p.synaptic_score * 9) / 10;
            }
        }
    }

    pub fn stored_peer_count(&self) -> usize {
        let axons: usize = self.axon_xor_buckets.iter().map(|b| b.len()).sum();
        self.intra_cluster_peers.len() + axons
    }

    pub fn active_xor_bucket_counts(&self) -> Vec<(usize, usize)> {
        self.axon_xor_buckets
            .iter()
            .enumerate()
            .filter(|(_, b)| !b.is_empty())
            .map(|(idx, b)| (idx, b.len()))
            .collect()
    }

    /// Exact in-memory footprint of all stored IP contacts in bytes.
    pub fn estimated_ram_bytes(&self) -> usize {
        self.stored_peer_count() * std::mem::size_of::<PeerContact>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skynet_dna::CellDna;

    #[test]
    fn fountain_codes_reconstruct_cell_dna_over_40_percent_udp_packet_loss() {
        let dna = CellDna::genesis_stem(2026);
        let raw_dna = bincode::serialize(&dna).unwrap();
        assert!(raw_dna.len() > MAX_UDP_DATAGRAM_BYTES);

        let key = SigningKey::from_bytes(&[7u8; 32]);
        let mut enc = FountainEncoder::new(&raw_dna).unwrap();
        let mut rng = ChaCha8Rng::seed_from_u64(12345);

        let mut decoder: Option<FountainDecoder> = None;
        let mut recovered: Option<Vec<u8>> = None;
        let mut sent = 0usize;
        let mut survived = 0usize;

        // Simulate aggressive 40% random UDP packet loss!
        while recovered.is_none() && sent < 100 {
            let droplet = enc.next_droplet();
            let pkt = ImpulsePacket::new_signed(
                &key,
                0,
                0,
                4,
                1,
                sent as u32,
                ImpulseBody::FountainDroplet(droplet),
            )
            .unwrap();
            let wire_bytes = pkt.encode_datagram().unwrap();
            assert!(wire_bytes.len() <= MAX_UDP_DATAGRAM_BYTES);
            sent += 1;

            // Drop 40% of UDP datagrams in transit!
            if rng.gen_bool(0.40) {
                continue;
            }
            survived += 1;

            let verified = ImpulsePacket::decode_and_verify(&wire_bytes).unwrap();
            if let ImpulseBody::FountainDroplet(d) = verified.body {
                let dec = decoder.get_or_insert_with(|| FountainDecoder::from_first_droplet(&d));
                recovered = dec.absorb(d).unwrap();
            }
        }

        let bytes = recovered.expect("must recover DNA despite 40% UDP loss");
        let decoded_dna: CellDna = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded_dna, dna);
        assert!(survived <= (enc.num_blocks() as usize) + 6);
    }

    #[test]
    fn tampered_udp_datagram_is_rejected() {
        let key = SigningKey::from_bytes(&[9u8; 32]);
        let pkt = ImpulsePacket::new_signed(
            &key,
            0b10,
            2,
            3,
            10,
            1,
            ImpulseBody::Heartbeat {
                tier_code: 2,
                total_cells: 500,
                stem_cells: 40,
                on_chain_nano_skn: 1_000_000_000,
                elite_dna_hash: [1u8; 32],
            },
        )
        .unwrap();
        let mut bytes = pkt.encode_datagram().unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        assert!(ImpulsePacket::decode_and_verify(&bytes).is_err());
    }

    #[test]
    fn synaptic_peer_table_bounds_10000_ips_under_6kb_ram_with_logarithmic_buckets() {
        let local_pub = [0xAAu8; 32];
        // TinyPeripheral limits: max 12 intra-cluster + max 2 per XOR distance bucket!
        let mut table = SynapticPeerTable::new(&local_pub, 0b01, 2, 12, 2);
        let mut rng = ChaCha8Rng::seed_from_u64(9999);

        for i in 0..10_000u32 {
            let mut pk = [0u8; 32];
            rng.fill(&mut pk);
            let port = (10_000 + (i % 50_000)) as u16;
            let addr: SocketAddr = format!("10.{}.{}.{}:{}", (i >> 16) & 255, (i >> 8) & 255, i & 255, port)
                .parse()
                .unwrap();
            table.observe_peer(PeerContact {
                pubkey: pk,
                addr,
                cluster_prefix: i % 4,
                prefix_len: 2,
                bonded_skn_whole: (i as u64 % 500) * 10,
                synaptic_score: 0,
            });
        }

        // Despite observing 10,000 distinct IPs (which naively costs ~720 KB),
        // TinyPeripheral stores <= 60 IPs (~4.3 KB RAM!) across logarithmic XOR buckets!
        assert_eq!(table.total_observed_ips_ever, 10_000);
        assert!(table.stored_peer_count() <= 64);
        assert!(table.estimated_ram_bytes() < 6_000);
        assert!(table.total_hebbian_evictions > 1_000);
    }
}
