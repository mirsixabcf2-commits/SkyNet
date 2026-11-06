//! Turnkey data preprocessing pipeline for text, tabular datasets, and cryptographic envelopes.

pub mod crypto_envelope;
pub mod tabular_csv;
pub mod text_pii;

pub use crypto_envelope::{decrypt_envelope, encrypt_envelope, generate_ephemeral_key};
pub use tabular_csv::{clean_csv, detect_delimiter, standardize_e164_vietnam, CleanCsvOutput, ImputeStrategy};
pub use text_pii::{redact_pii, sliding_window_chunks};

use serde::{Deserialize, Serialize};
use skynet_cell::CandleTaskInput;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessOptions {
    pub redact_pii: bool,
    pub chunk_sliding_window: Option<(usize, usize)>,
    pub csv_clean: bool,
    pub csv_impute: ImputeStrategy,
    pub csv_normalize_minmax: bool,
    pub standardize_e164: bool,
    pub encrypt_envelope: bool,
}

impl Default for PreprocessOptions {
    fn default() -> Self {
        Self {
            redact_pii: true,
            chunk_sliding_window: None,
            csv_clean: false,
            csv_impute: ImputeStrategy::Mean,
            csv_normalize_minmax: true,
            standardize_e164: true,
            encrypt_envelope: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessResult {
    pub processed_text: Option<String>,
    pub chunks: Vec<String>,
    pub tabular_matrix: Option<CleanCsvOutput>,
    pub envelope_bytes: Option<Vec<u8>>,
    pub encryption_key_hex: Option<String>,
    pub tensor_floats: Vec<f32>,
    pub raw_bytes_count: usize,
    pub processed_bytes_count: usize,
}

/// Executes configured preprocessing routines across textual or tabular inputs.
pub fn run_preprocessing(input: &str, opts: &PreprocessOptions) -> Result<PreprocessResult, String> {
    let raw_bytes_count = input.len();
    let mut current_text = input.to_string();
    let mut tabular_output = None;

    if opts.standardize_e164 {
        current_text = apply_e164_batch(&current_text);
    }

    if opts.redact_pii {
        current_text = redact_pii(&current_text);
    }

    let chunks = if let Some((win, overlap)) = opts.chunk_sliding_window {
        sliding_window_chunks(&current_text, win, overlap)
    } else {
        Vec::new()
    };

    let mut tensor_floats = Vec::new();
    if opts.csv_clean {
        let clean_csv_res = clean_csv(&current_text, opts.csv_impute, opts.csv_normalize_minmax)?;
        for row in &clean_csv_res.data {
            tensor_floats.extend_from_slice(row);
        }
        tabular_output = Some(clean_csv_res);
    } else {
        // Fallback: character frequency or ASCII byte floats
        tensor_floats = current_text
            .bytes()
            .take(512)
            .map(|b| (b as f32) / 255.0)
            .collect();
    }

    let (envelope_bytes, encryption_key_hex) = if opts.encrypt_envelope {
        let key = generate_ephemeral_key();
        let encrypted = encrypt_envelope(current_text.as_bytes(), &key);
        let key_hex = key.iter().map(|b| format!("{b:02x}")).collect::<String>();
        (Some(encrypted), Some(key_hex))
    } else {
        (None, None)
    };

    let processed_bytes_count = if let Some(ref env) = envelope_bytes {
        env.len()
    } else {
        current_text.len()
    };

    Ok(PreprocessResult {
        processed_text: Some(current_text),
        chunks,
        tabular_matrix: tabular_output,
        envelope_bytes,
        encryption_key_hex,
        tensor_floats,
        raw_bytes_count,
        processed_bytes_count,
    })
}

fn apply_e164_batch(text: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    for token in text.split_whitespace() {
        if let Some(standardized) = standardize_e164_vietnam(token) {
            words.push(standardized);
        } else {
            words.push(token.to_string());
        }
    }
    words.join(" ")
}

/// Helper to wrap preprocessed numerical tensors into an executable `CandleTaskInput`.
pub fn to_candle_task(
    tensor_floats: &[f32],
    dim_in: usize,
    dim_hidden: usize,
    dim_out: usize,
) -> CandleTaskInput {
    let mut input = tensor_floats.to_vec();
    if input.len() < dim_in {
        input.resize(dim_in, 0.0);
    } else if input.len() > dim_in {
        input.truncate(dim_in);
    }

    let dummy_weights_len = dim_in * dim_hidden + dim_hidden + dim_hidden * dim_out + dim_out;
    let weights = vec![0.01f32; dummy_weights_len];

    let mut task_id = [0u8; 32];
    let digest = blake3::hash(&(tensor_floats.len() as u64).to_le_bytes());
    task_id.copy_from_slice(digest.as_bytes());

    CandleTaskInput {
        task_id,
        dim_in,
        dim_hidden,
        dim_out,
        weights,
        input,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_text_pipeline() {
        let text = "Contact anh Minh qua 0987654321 hoac email minh@cty.vn.";
        let opts = PreprocessOptions {
            redact_pii: true,
            chunk_sliding_window: Some((3, 1)),
            csv_clean: false,
            csv_impute: ImputeStrategy::Mean,
            csv_normalize_minmax: false,
            standardize_e164: true,
            encrypt_envelope: true,
        };

        let res = run_preprocessing(text, &opts).expect("Preprocessing failed");
        let processed = res.processed_text.as_ref().unwrap();
        assert!(processed.contains("[REDACTED_PHONE]"));
        assert!(processed.contains("[REDACTED_EMAIL]"));
        assert!(res.envelope_bytes.is_some());
        assert!(res.encryption_key_hex.is_some());
        assert!(!res.chunks.is_empty());
    }

    #[test]
    fn test_candle_task_conversion() {
        let floats = vec![0.1, 0.2, 0.3, 0.4];
        let task = to_candle_task(&floats, 4, 8, 2);
        assert_eq!(task.dim_in, 4);
        assert_eq!(task.dim_hidden, 8);
        assert_eq!(task.dim_out, 2);
        assert_eq!(task.input, floats);
    }
}
