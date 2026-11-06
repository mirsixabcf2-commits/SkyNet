//! `fountain`: ACK-less GF(2) Fountain erasure code encoder, decoder, and droplets
//! allowing reconstruction of multi-KB payloads over lossy connectionless UDP.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::WireError;

pub const FOUNTAIN_BLOCK_BYTES: usize = 512;

/// Single erasure-coded droplet (`~560 bytes`) for ACK-less transfer over lossy UDP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
