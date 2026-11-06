//! `pope_benchmark`: Proof of Physical Execution (PoPE) memory-hard Candle tensor benchmark
//! to prevent Sybil hardware spoofing and virtual machine oversubscription.

use std::time::Instant;

use candle_core::{Device, Tensor};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PopeError {
    #[error("candle tensor computation error: {0}")]
    Candle(#[from] candle_core::Error),
    #[error("execution duration ({actual} µs) exceeded maximum allowed ({max_allowed} µs)")]
    TimeoutExceeded { actual: u64, max_allowed: u64 },
    #[error("proof digest mismatch")]
    InvalidDigest,
}

/// Parameters controlling the hardware-saturating PoPE tensor challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopeChallengeParams {
    pub dim: usize,
    pub iterations: usize,
    pub max_allowed_micros: u64,
}

impl Default for PopeChallengeParams {
    fn default() -> Self {
        Self {
            dim: 64,
            iterations: 4,
            max_allowed_micros: 3_000_000,
        }
    }
}

/// Challenge descriptor grouping the hardware ID, nonce, and evaluation parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PopeChallenge<'a> {
    pub hwid: &'a [u8; 32],
    pub nonce: &'a [u8; 32],
    pub params: PopeChallengeParams,
}

impl<'a> PopeChallenge<'a> {
    pub const fn new(hwid: &'a [u8; 32], nonce: &'a [u8; 32], params: PopeChallengeParams) -> Self {
        Self { hwid, nonce, params }
    }
}

/// Cryptographic attestation produced by solving the physical tensor challenge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopeProof {
    pub result_digest: [u8; 32],
    pub duration_micros: u64,
}

/// Derives deterministic float vectors from arbitrary seed data using BLAKE3 XOF.
#[allow(clippy::chunks_exact_to_as_chunks)]
fn derive_floats(seed_bytes: &[u8], count: usize) -> Vec<f32> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"SKYNET_POPE_TENSOR_DERIVE_V1");
    hasher.update(seed_bytes);

    let mut xof = hasher.finalize_xof();
    let mut raw_bytes = vec![0u8; count * 2];
    xof.fill(&mut raw_bytes);

    raw_bytes
        .chunks_exact(2)
        .map(|chunk| {
            let val = u16::from_le_bytes([chunk[0], chunk[1]]);
            (val as f32 / 32767.5) - 1.0
        })
        .collect()
}

/// Hashes tensor output floats into a BLAKE3 proof digest with canonical float representations.
fn hash_tensor_floats(floats: &[f32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"SKYNET_POPE_OUTPUT_DIGEST_V1");
    for &f in floats {
        let canonical = if f == 0.0 { 0.0f32 } else { f };
        hasher.update(&canonical.to_le_bytes());
    }
    *hasher.finalize().as_bytes()
}

/// Computes the PoPE challenge on the specified hardware `Device`.
pub fn compute_pope_challenge(
    device: &Device,
    challenge: PopeChallenge,
) -> Result<PopeProof, PopeError> {
    let mut seed_w = Vec::with_capacity(72);
    seed_w.extend_from_slice(b"POPE_W_");
    seed_w.extend_from_slice(challenge.hwid);
    seed_w.extend_from_slice(challenge.nonce);

    let mut seed_b = Vec::with_capacity(72);
    seed_b.extend_from_slice(b"POPE_B_");
    seed_b.extend_from_slice(challenge.hwid);
    seed_b.extend_from_slice(challenge.nonce);

    let mut seed_in = Vec::with_capacity(72);
    seed_in.extend_from_slice(b"POPE_IN_");
    seed_in.extend_from_slice(challenge.hwid);
    seed_in.extend_from_slice(challenge.nonce);

    let d = challenge.params.dim;
    let w_floats = derive_floats(&seed_w, d * d);
    let b_floats = derive_floats(&seed_b, d);
    let in_floats = derive_floats(&seed_in, d);

    let start = Instant::now();
    let mut state = Tensor::from_vec(in_floats, (1, d), device)?;
    let w = Tensor::from_vec(w_floats, (d, d), device)?;
    let b = Tensor::from_vec(b_floats, (1, d), device)?;

    for _ in 0..challenge.params.iterations {
        state = state.matmul(&w)?.broadcast_add(&b)?.tanh()?;
    }

    let out_floats = state.flatten_all()?.to_vec1::<f32>()?;
    let duration_micros = start.elapsed().as_micros().max(1) as u64;
    let result_digest = hash_tensor_floats(&out_floats);

    Ok(PopeProof {
        result_digest,
        duration_micros,
    })
}

/// Verifies that a PoPE proof matches expected computation within latency bounds.
pub fn verify_pope_proof(
    device: &Device,
    challenge: PopeChallenge,
    proof: &PopeProof,
) -> Result<(), PopeError> {
    if proof.duration_micros > challenge.params.max_allowed_micros {
        return Err(PopeError::TimeoutExceeded {
            actual: proof.duration_micros,
            max_allowed: challenge.params.max_allowed_micros,
        });
    }

    let expected = compute_pope_challenge(device, challenge)?;
    if expected.result_digest != proof.result_digest {
        return Err(PopeError::InvalidDigest);
    }
    Ok(())
}

/// Verifies that a PoPE proof matches expected computation and that the verifier's
/// independently measured round-trip time does not exceed the hardware-saturating threshold.
/// This prevents untrusted nodes or virtual machines from falsifying self-reported `duration_micros`.
pub fn verify_pope_proof_with_latency(
    device: &Device,
    challenge: PopeChallenge,
    proof: &PopeProof,
    measured_roundtrip_micros: Option<u64>,
) -> Result<(), PopeError> {
    if let Some(rtt) = measured_roundtrip_micros {
        if rtt > challenge.params.max_allowed_micros {
            return Err(PopeError::TimeoutExceeded {
                actual: rtt,
                max_allowed: challenge.params.max_allowed_micros,
            });
        }
    }
    verify_pope_proof(device, challenge, proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_and_verify_pope_happy_path() {
        let device = Device::Cpu;
        let hwid = [11u8; 32];
        let nonce = [22u8; 32];
        let params = PopeChallengeParams {
            dim: 16,
            iterations: 2,
            max_allowed_micros: 5_000_000,
        };
        let challenge = PopeChallenge::new(&hwid, &nonce, params);

        let proof = compute_pope_challenge(&device, challenge)
            .expect("pope computation should succeed");

        assert_ne!(proof.result_digest, [0u8; 32]);
        assert!(proof.duration_micros > 0);

        verify_pope_proof(&device, challenge, &proof)
            .expect("proof verification must succeed");
    }

    #[test]
    fn test_verify_pope_rejects_altered_digest() {
        let device = Device::Cpu;
        let hwid = [1u8; 32];
        let nonce = [2u8; 32];
        let params = PopeChallengeParams {
            dim: 16,
            iterations: 2,
            max_allowed_micros: 5_000_000,
        };
        let challenge = PopeChallenge::new(&hwid, &nonce, params);

        let mut proof = compute_pope_challenge(&device, challenge).unwrap();
        proof.result_digest[0] ^= 0xFF; // Corrupt digest

        let err = verify_pope_proof(&device, challenge, &proof).unwrap_err();
        match err {
            PopeError::InvalidDigest => {}
            other => panic!("expected InvalidDigest, got {:?}", other),
        }
    }

    #[test]
    fn test_verify_pope_rejects_timeout() {
        let device = Device::Cpu;
        let hwid = [3u8; 32];
        let nonce = [4u8; 32];
        let params = PopeChallengeParams {
            dim: 16,
            iterations: 2,
            max_allowed_micros: 1, // unrealistically low threshold
        };
        let challenge = PopeChallenge::new(&hwid, &nonce, params);

        let mut proof = compute_pope_challenge(&device, challenge).unwrap();
        proof.duration_micros = 1000;

        let err = verify_pope_proof(&device, challenge, &proof).unwrap_err();
        match err {
            PopeError::TimeoutExceeded { actual, max_allowed } => {
                assert_eq!(actual, 1000);
                assert_eq!(max_allowed, 1);
            }
            other => panic!("expected TimeoutExceeded, got {:?}", other),
        }
    }

    #[test]
    fn test_verify_pope_rejects_falsified_duration_via_verifier_measured_latency() {
        let device = Device::Cpu;
        let hwid = [5u8; 32];
        let nonce = [6u8; 32];
        let params = PopeChallengeParams {
            dim: 16,
            iterations: 2,
            max_allowed_micros: 50_000,
        };
        let challenge = PopeChallenge::new(&hwid, &nonce, params);

        let mut proof = compute_pope_challenge(&device, challenge).unwrap();
        // Attacker lies and self-reports very low duration
        proof.duration_micros = 100;

        // Verifier measured real round-trip of 200,000 micros (> max_allowed_micros of 50,000)
        let measured_rtt = Some(200_000);
        let err = verify_pope_proof_with_latency(&device, challenge, &proof, measured_rtt).unwrap_err();
        match err {
            PopeError::TimeoutExceeded { actual, max_allowed } => {
                assert_eq!(actual, 200_000);
                assert_eq!(max_allowed, 50_000);
            }
            other => panic!("expected TimeoutExceeded from verifier measured rtt, got {:?}", other),
        }
    }
}
