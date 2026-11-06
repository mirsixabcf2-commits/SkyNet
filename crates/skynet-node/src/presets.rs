//! Catalog and factory for turnkey task preprocessing presets.

use serde::{Deserialize, Serialize};
use crate::preprocessor::{ImputeStrategy, PreprocessOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnkeyPreset {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub default_opcode: u16,
    pub suggested_bounty_skn: u64,
    pub category: &'static str,
}

pub const TURNKEY_PRESETS: &[TurnkeyPreset] = &[
    TurnkeyPreset {
        id: "text-audit",
        title: "Kiem Toan Van Ban Bao Mat (PII Scrubbing & Audit)",
        description: "Tu dong che giau thong tin ca nhan (SDT, Email, CCCD, Key) va cat lat van ban",
        default_opcode: 0x0215,
        suggested_bounty_skn: 15,
        category: "Security & NLP",
    },
    TurnkeyPreset {
        id: "csv-clean-normalize",
        title: "Lam Sach & Chuan Hoa Bang Tinh CSV (Tabular ML)",
        description: "Nhan dien delimiter, dien gia tri thieu (Mean) va Min-Max scaling [0..1]",
        default_opcode: 0x0210,
        suggested_bounty_skn: 10,
        category: "Data Science & ETL",
    },
    TurnkeyPreset {
        id: "image-batch-upscale",
        title: "Nang Cao Do Phan Giai Anh Hang Loat (Real-ESRGAN)",
        description: "Quet thu muc anh va phan chia theo lo de sieu phan giai 4x",
        default_opcode: 0x0200,
        suggested_bounty_skn: 25,
        category: "Computer Vision",
    },
    TurnkeyPreset {
        id: "log-anomaly-detection",
        title: "Phat Hien Di Thuong Trong Log He Thong",
        description: "Lam sach log su kien va trich xuat vector tan suat phat hien bat thuong",
        default_opcode: 0x0220,
        suggested_bounty_skn: 12,
        category: "DevOps & Security",
    },
    TurnkeyPreset {
        id: "phone-standardize",
        title: "Chuan Hoa Danh Ba CRM Sang Chuan E.164 Quoc Te",
        description: "Chuyen doi so noi dia Viet Nam sang +84 va loai bo ky tu rac",
        default_opcode: 0x0230,
        suggested_bounty_skn: 5,
        category: "Data Cleaning",
    },
];

/// Returns the full list of standardized turnkey presets.
pub fn list_presets() -> &'static [TurnkeyPreset] {
    TURNKEY_PRESETS
}

/// Looks up a preset by its unique string identifier.
pub fn find_preset(id: &str) -> Option<&'static TurnkeyPreset> {
    TURNKEY_PRESETS.iter().find(|p| p.id.eq_ignore_ascii_case(id))
}

/// Generates the recommended preprocessing options for a given preset ID.
pub fn default_options_for_preset(id: &str) -> PreprocessOptions {
    match id {
        "text-audit" => PreprocessOptions {
            redact_pii: true,
            chunk_sliding_window: Some((256, 32)),
            csv_clean: false,
            csv_impute: ImputeStrategy::Mean,
            csv_normalize_minmax: false,
            standardize_e164: true,
            encrypt_envelope: true,
        },
        "csv-clean-normalize" => PreprocessOptions {
            redact_pii: false,
            chunk_sliding_window: None,
            csv_clean: true,
            csv_impute: ImputeStrategy::Mean,
            csv_normalize_minmax: true,
            standardize_e164: false,
            encrypt_envelope: false,
        },
        "phone-standardize" => PreprocessOptions {
            redact_pii: false,
            chunk_sliding_window: None,
            csv_clean: false,
            csv_impute: ImputeStrategy::Zero,
            csv_normalize_minmax: false,
            standardize_e164: true,
            encrypt_envelope: false,
        },
        _ => PreprocessOptions::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_presets() {
        let preset = find_preset("text-audit").expect("Preset not found");
        assert_eq!(preset.default_opcode, 0x0215);
        assert_eq!(preset.suggested_bounty_skn, 15);

        let csv = find_preset("csv-clean-normalize").expect("Preset not found");
        assert_eq!(csv.default_opcode, 0x0210);

        let none = find_preset("nonexistent-preset");
        assert!(none.is_none());
    }

    #[test]
    fn test_default_options() {
        let opts = default_options_for_preset("text-audit");
        assert!(opts.redact_pii);
        assert!(opts.encrypt_envelope);
        assert!(opts.chunk_sliding_window.is_some());
    }
}
