//! `utreexo`: Stateless `~1 KB` Binary Merkle Forest accumulator and inclusion proofs.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{LedgerError, TxOutput, UtxoId};

/// Compact Merkle Inclusion Proof (`~64 - 384 bytes`) for stateless UTXO verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtreexoProof {
    pub leaf_position: u64,
    pub tree_height: u8,
    pub siblings: Vec<[u8; 32]>,
}

/// Ultra-lightweight **Utreexo Binary Merkle Forest Accumulator** (`<= 2 KB` RAM total).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatelessUtreexoRoots {
    pub num_leaves: u64,
    /// Active roots `(height, root_hash)` — typically `<= 32` entries (`~1 KB` RAM).
    pub forest_roots: Vec<(u8, [u8; 32])>,
}

impl Default for StatelessUtreexoRoots {
    fn default() -> Self {
        Self::new()
    }
}

impl StatelessUtreexoRoots {
    pub fn new() -> Self {
        Self {
            num_leaves: 0,
            forest_roots: Vec::new(),
        }
    }

    /// Exact RAM footprint of the active cryptographic roots in bytes (`~33 bytes * popcount`).
    pub fn ram_footprint_bytes(&self) -> usize {
        std::mem::size_of::<u64>() + self.forest_roots.len() * 33
    }

    pub fn hash_utxo_leaf(id: &UtxoId, out: &TxOutput) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"SKYNET_UTREEXO_LEAF_V1");
        h.update(&id.tx_hash);
        h.update(&id.vout.to_le_bytes());
        out.hash_into(&mut h);
        *h.finalize().as_bytes()
    }

    pub fn hash_parent(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut h = blake3::Hasher::new();
        h.update(b"SKYNET_UTREEXO_NODE_V1");
        h.update(left);
        h.update(right);
        *h.finalize().as_bytes()
    }

    /// Appends a new UTXO leaf in $O(\log N)$ time using `< 2 KB` of state.
    pub fn add_leaf(&mut self, mut carry: [u8; 32]) {
        let mut h = 0u8;
        while let Some(&(_, prev_root)) = self.forest_roots.iter().find(|&&(ht, _)| ht == h) {
            self.forest_roots.retain(|&(ht, _)| ht != h);
            carry = Self::hash_parent(&prev_root, &carry);
            h += 1;
        }
        self.forest_roots.push((h, carry));
        self.forest_roots.sort_by_key(|&(ht, _)| ht);
        self.num_leaves += 1;
    }

    /// Statelessly verifies a UTXO's existence in `< 10 microseconds` using ONLY `~1 KB` of roots.
    pub fn verify_utxo(
        &self,
        id: &UtxoId,
        out: &TxOutput,
        proof: &UtreexoProof,
    ) -> Result<(), LedgerError> {
        if proof.siblings.len() != proof.tree_height as usize {
            return Err(LedgerError::InvalidUtreexoProof);
        }
        let expected_root = self
            .forest_roots
            .iter()
            .find(|&&(ht, _)| ht == proof.tree_height)
            .map(|&(_, r)| r)
            .ok_or(LedgerError::InvalidUtreexoProof)?;

        let mut curr = Self::hash_utxo_leaf(id, out);
        let mut pos = proof.leaf_position;
        for sib in &proof.siblings {
            curr = if pos & 1 == 0 {
                Self::hash_parent(&curr, sib)
            } else {
                Self::hash_parent(sib, &curr)
            };
            pos >>= 1;
        }

        if curr != expected_root {
            return Err(LedgerError::InvalidUtreexoProof);
        }
        Ok(())
    }
}

/// Builds Utreexo proofs from an active UTXO snapshot (used by wallets / cluster provers).
pub struct UtreexoForestBuilder {
    leaves: Vec<[u8; 32]>,
    index_of: HashMap<UtxoId, usize>,
    pub roots: StatelessUtreexoRoots,
}

impl UtreexoForestBuilder {
    pub fn from_utxos<'a>(utxos: impl IntoIterator<Item = (&'a UtxoId, &'a TxOutput)>) -> Self {
        let mut leaves = Vec::new();
        let mut index_of = HashMap::new();
        let mut roots = StatelessUtreexoRoots::new();

        for (i, (id, out)) in utxos.into_iter().enumerate() {
            let leaf = StatelessUtreexoRoots::hash_utxo_leaf(id, out);
            leaves.push(leaf);
            index_of.insert(*id, i);
            roots.add_leaf(leaf);
        }

        Self {
            leaves,
            index_of,
            roots,
        }
    }

    fn build_subtree_proof(slice: &[[u8; 32]], local_idx: usize, shift: usize) -> UtreexoProof {
        let mut level = slice.to_vec();
        let mut siblings = Vec::with_capacity(shift);
        let mut cur = local_idx;
        while level.len() > 1 {
            siblings.push(level[cur ^ 1]);
            let mut next_level = Vec::with_capacity(level.len() / 2);
            #[allow(clippy::chunks_exact_to_as_chunks)]
            for pair in level.chunks_exact(2) {
                next_level.push(StatelessUtreexoRoots::hash_parent(&pair[0], &pair[1]));
            }
            level = next_level;
            cur >>= 1;
        }
        UtreexoProof {
            leaf_position: local_idx as u64,
            tree_height: shift as u8,
            siblings,
        }
    }

    pub fn prove(&self, id: &UtxoId) -> Option<UtreexoProof> {
        let global_idx = *self.index_of.get(id)?;
        let n = self.leaves.len();
        let mut offset = 0usize;

        for shift in (0..64).rev() {
            let size = 1usize << shift;
            if (n & size) == 0 {
                continue;
            }
            if global_idx >= offset && global_idx < offset + size {
                let local_idx = global_idx - offset;
                return Some(Self::build_subtree_proof(
                    &self.leaves[offset..offset + size],
                    local_idx,
                    shift,
                ));
            }
            offset += size;
        }
        None
    }
}
