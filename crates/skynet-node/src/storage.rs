//! Atomic file-based persistence engine for node state, ledger, cells, and peer table.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use skynet_agent::AgentNode;
use skynet_ledger::UtxoDagLedger;
use skynet_wire::{PeerContact, SynapticPeerTable};
use thiserror::Error;

use crate::storage_types::{
    LedgerSnapshot, NodeMetadata, OrganismSnapshot, PeerTableSnapshot,
};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("filesystem I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode serialization error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("corrupt storage state: {0}")]
    CorruptState(String),
}

/// Provides atomic load and save operations for node daemon persistence.
#[derive(Clone, Debug)]
pub struct StorageEngine {
    data_dir: PathBuf,
}

impl StorageEngine {
    pub fn new(data_dir: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let dir = data_dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self { data_dir: dir })
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn metadata_path(&self) -> PathBuf {
        self.data_dir.join("metadata.json")
    }

    pub fn ledger_path(&self) -> PathBuf {
        self.data_dir.join("ledger.dat")
    }

    pub fn organism_path(&self) -> PathBuf {
        self.data_dir.join("organism.dat")
    }

    pub fn peers_path(&self) -> PathBuf {
        self.data_dir.join("peers.json")
    }

    pub fn has_existing_state(&self) -> bool {
        self.metadata_path().exists() && self.organism_path().exists()
    }

    /// Atomically writes bytes to disk by writing to a .tmp file then renaming.
    pub fn atomic_write(target: &Path, bytes: &[u8]) -> Result<(), StorageError> {
        let tmp = target.with_extension("tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(bytes)?;
            file.flush()?;
            file.sync_all()?;
        }
        fs::rename(&tmp, target)?;
        Ok(())
    }

    pub fn save_metadata(&self, meta: &NodeMetadata) -> Result<(), StorageError> {
        let data = serde_json::to_vec_pretty(meta)?;
        Self::atomic_write(&self.metadata_path(), &data)
    }

    pub fn load_metadata(&self) -> Result<Option<NodeMetadata>, StorageError> {
        let path = self.metadata_path();
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path)?;
        let meta = serde_json::from_slice(&bytes)?;
        Ok(Some(meta))
    }

    pub fn save_ledger(&self, ledger: &UtxoDagLedger) -> Result<(), StorageError> {
        let snap = LedgerSnapshot {
            utxos: ledger.all_utxos(),
            finalized_tips: ledger.finalized_tips(),
            cluster_nutrient_pool: ledger.cluster_nutrient_pool,
        };
        let bytes = bincode::serialize(&snap)?;
        Self::atomic_write(&self.ledger_path(), &bytes)
    }

    pub fn load_ledger(&self) -> Result<Option<UtxoDagLedger>, StorageError> {
        let path = self.ledger_path();
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path)?;
        let snap: LedgerSnapshot = bincode::deserialize(&bytes)?;
        Ok(Some(UtxoDagLedger::from_raw_state(
            snap.utxos,
            snap.finalized_tips,
            snap.cluster_nutrient_pool,
        )))
    }

    pub fn save_organism(&self, agent: &AgentNode) -> Result<(), StorageError> {
        let snap = OrganismSnapshot {
            cells: agent.cells.clone(),
            blood_pool: agent.bank.blood_pool,
            total_voucher_nano_earned: agent.total_voucher_nano_earned,
        };
        let bytes = bincode::serialize(&snap)?;
        Self::atomic_write(&self.organism_path(), &bytes)
    }

    pub fn load_organism(&self) -> Result<Option<OrganismSnapshot>, StorageError> {
        let path = self.organism_path();
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path)?;
        let snap: OrganismSnapshot = bincode::deserialize(&bytes)?;
        Ok(Some(snap))
    }

    pub fn save_peers(&self, peer_table: &SynapticPeerTable) -> Result<(), StorageError> {
        let mut peers = peer_table.intra_cluster_peers.clone();
        for b in &peer_table.axon_xor_buckets {
            peers.extend(b.iter().cloned());
        }
        let snap = PeerTableSnapshot { peers };
        let bytes = serde_json::to_vec_pretty(&snap)?;
        Self::atomic_write(&self.peers_path(), &bytes)
    }

    pub fn load_peers(&self) -> Result<Vec<PeerContact>, StorageError> {
        let path = self.peers_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let bytes = fs::read(&path)?;
        let snap: PeerTableSnapshot = serde_json::from_slice(&bytes)?;
        Ok(snap.peers)
    }

    pub fn save_all(&self, agent: &AgentNode) -> Result<(), StorageError> {
        let meta = NodeMetadata {
            name: agent.name.clone(),
            tier: agent.profile.tier,
            pubkey: agent.pubkey,
            epoch: agent.epoch,
            pulse_seq: agent.pulse_seq,
            created_at_secs: 0,
            last_saved_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };
        self.save_metadata(&meta)?;
        self.save_ledger(&agent.ledger)?;
        self.save_organism(agent)?;
        self.save_peers(&agent.peer_table)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand_chacha::rand_core::{OsRng, RngCore};
    use skynet_agent::SnowballParams;
    use skynet_cell::HardwareTier;
    use skynet_ledger::{NanoSkn, TxOutput};

    #[test]
    fn test_atomic_write_and_storage_save_load() {
        let tmp_dir = std::env::temp_dir().join(format!("skynet_storage_test_{}", OsRng.next_u64()));
        let storage = StorageEngine::new(&tmp_dir).expect("storage init must succeed");

        assert!(!storage.has_existing_state());

        let signing_key = SigningKey::generate(&mut OsRng);
        let pubkey = signing_key.verifying_key().to_bytes();
        let alloc = NanoSkn::from_skn(500).unwrap();

        let (ledger, _) = UtxoDagLedger::from_genesis(vec![TxOutput::direct(pubkey, alloc)]).unwrap();

        let mut agent = AgentNode::from_signing_key(
            "Test-Storage-Agent",
            42,
            signing_key,
            HardwareTier::StandardNode,
            ledger,
            SnowballParams::default(),
        )
        .unwrap();

        let stim = [0.1; 8];
        let target = [0.2; 4];
        agent.run_biological_tick(&stim, &target).unwrap();

        storage.save_all(&agent).expect("save_all must succeed");
        assert!(storage.has_existing_state());

        let loaded_meta = storage.load_metadata().unwrap().expect("meta should exist");
        assert_eq!(loaded_meta.name, "Test-Storage-Agent");
        assert_eq!(loaded_meta.pubkey, agent.pubkey);

        let loaded_ledger = storage.load_ledger().unwrap().expect("ledger should exist");
        assert_eq!(loaded_ledger.balance_of(&agent.pubkey), alloc);

        let loaded_org = storage.load_organism().unwrap().expect("organism should exist");
        assert_eq!(loaded_org.cells.len(), agent.cells.len());
        assert_eq!(loaded_org.blood_pool, agent.bank.blood_pool);

        let _ = std::fs::remove_dir_all(tmp_dir);
    }
}

