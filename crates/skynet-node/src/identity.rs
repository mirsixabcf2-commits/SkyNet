//! Node identity management and Ed25519 key persistence with secure filesystem permissions.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use ed25519_dalek::SigningKey;
use rand_chacha::rand_core::OsRng;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("filesystem I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid identity key length: expected 32 bytes, got {0}")]
    InvalidKeyLength(usize),
}

/// Represents the cryptographic identity of a SkyNet node.
#[derive(Clone)]
pub struct NodeIdentity {
    pub signing_key: SigningKey,
    pub pubkey: [u8; 32],
}

impl NodeIdentity {
    /// Loads an existing Ed25519 signing key or generates a new one securely.
    pub fn load_or_generate(key_path: &Path) -> Result<Self, IdentityError> {
        if key_path.exists() {
            Self::load_from_file(key_path)
        } else {
            Self::generate_and_save(key_path)
        }
    }

    fn load_from_file(path: &Path) -> Result<Self, IdentityError> {
        let mut file = File::open(path)?;
        let mut secret_bytes = Vec::new();
        file.read_to_end(&mut secret_bytes)?;
        if secret_bytes.len() != 32 {
            return Err(IdentityError::InvalidKeyLength(secret_bytes.len()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&secret_bytes);
        let signing_key = SigningKey::from_bytes(&arr);
        let pubkey = signing_key.verifying_key().to_bytes();
        Ok(Self { signing_key, pubkey })
    }

    fn generate_and_save(path: &Path) -> Result<Self, IdentityError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let signing_key = SigningKey::generate(&mut OsRng);
        let secret_bytes = signing_key.to_bytes();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(path, perms)?;
        }

        file.write_all(&secret_bytes)?;
        file.flush()?;

        let pubkey = signing_key.verifying_key().to_bytes();
        Ok(Self { signing_key, pubkey })
    }

    pub fn pubkey_hex(&self) -> String {
        self.pubkey.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_chacha::rand_core::RngCore;

    #[test]
    fn test_load_or_generate_creates_and_reloads_same_identity() {
        let tmp_dir = std::env::temp_dir().join(format!("skynet_id_test_{}", OsRng.next_u64()));
        let key_path = tmp_dir.join("node_identity.key");

        // 1. Generate initially
        let id1 = NodeIdentity::load_or_generate(&key_path).expect("generate must succeed");
        assert!(key_path.exists());
        assert_eq!(id1.pubkey.len(), 32);

        // 2. Reload existing
        let id2 = NodeIdentity::load_or_generate(&key_path).expect("reload must succeed");
        assert_eq!(id1.pubkey, id2.pubkey);
        assert_eq!(id1.signing_key.to_bytes(), id2.signing_key.to_bytes());

        let _ = std::fs::remove_dir_all(tmp_dir);
    }
}

