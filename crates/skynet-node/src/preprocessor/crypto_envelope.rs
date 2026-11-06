//! Cryptographic envelope encryption for privacy-preserving distributed task dispatch.

use rand::RngCore;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 32;

/// Generates a cryptographically secure 256-bit ephemeral key for symmetric task encryption.
pub fn generate_ephemeral_key() -> [u8; 32] {
    let mut rng = ChaCha8Rng::from_entropy();
    let mut key = [0u8; 32];
    rng.fill_bytes(&mut key);
    key
}

/// Encrypts a payload into an authenticated symmetric envelope:
/// Envelope structure: `[12-byte nonce] [32-byte blake3 tag] [ciphertext]`.
pub fn encrypt_envelope(payload: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let mut rng = ChaCha8Rng::from_entropy();
    let mut nonce = [0u8; NONCE_LEN];
    rng.fill_bytes(&mut nonce);

    let ciphertext = apply_stream_keystream(payload, key, &nonce);
    let tag = compute_auth_tag(key, &nonce, &ciphertext);

    let mut envelope = Vec::with_capacity(NONCE_LEN + TAG_LEN + ciphertext.len());
    envelope.extend_from_slice(&nonce);
    envelope.extend_from_slice(&tag);
    envelope.extend_from_slice(&ciphertext);
    envelope
}

/// Decrypts and verifies the authenticated symmetric envelope.
pub fn decrypt_envelope(envelope: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if envelope.len() < NONCE_LEN + TAG_LEN {
        return Err("Envelope too short to contain nonce and auth tag".to_string());
    }

    let nonce = &envelope[..NONCE_LEN];
    let tag = &envelope[NONCE_LEN..NONCE_LEN + TAG_LEN];
    let ciphertext = &envelope[NONCE_LEN + TAG_LEN..];

    let expected_tag = compute_auth_tag(key, nonce, ciphertext);
    if tag != expected_tag {
        return Err("Cryptographic authentication tag mismatch (tampered or wrong key)".to_string());
    }

    let plaintext = apply_stream_keystream(ciphertext, key, nonce);
    Ok(plaintext)
}

fn compute_auth_tag(key: &[u8; 32], nonce: &[u8], ciphertext: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_keyed(key);
    hasher.update(nonce);
    hasher.update(ciphertext);
    *hasher.finalize().as_bytes()
}

fn apply_stream_keystream(data: &[u8], key: &[u8; 32], nonce: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len());
    let chunk_size = 32;

    for (chunk_idx, chunk) in data.chunks(chunk_size).enumerate() {
        let mut key_block_hasher = blake3::Hasher::new_keyed(key);
        key_block_hasher.update(nonce);
        key_block_hasher.update(&(chunk_idx as u64).to_le_bytes());
        let block_digest = key_block_hasher.finalize();
        let keystream = block_digest.as_bytes();

        for (b, &k) in chunk.iter().zip(keystream.iter()) {
            output.push(b ^ k);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_roundtrip() {
        let key = generate_ephemeral_key();
        let original_data = b"Confidential financial statement and customer records.";

        let encrypted = encrypt_envelope(original_data, &key);
        assert_ne!(encrypted.as_slice(), original_data);

        let decrypted = decrypt_envelope(&encrypted, &key).expect("Decryption failed");
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_envelope_tamper_detection() {
        let key = generate_ephemeral_key();
        let mut encrypted = encrypt_envelope(b"Sensitive payload", &key);

        // Tamper with one byte in the ciphertext
        let last_idx = encrypted.len() - 1;
        encrypted[last_idx] ^= 0xFF;

        let res = decrypt_envelope(&encrypted, &key);
        assert!(res.is_err());
    }
}
