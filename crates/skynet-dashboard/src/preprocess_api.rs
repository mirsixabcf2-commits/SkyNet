//! Preprocessing API handlers and request parsing for SkyNet Dashboard.

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PreprocessApiReq {
    pub preset: Option<String>,
    pub input: Option<String>,
    pub redact_pii: Option<bool>,
    pub normalize_minmax: Option<bool>,
    pub standardize_e164: Option<bool>,
    pub encrypt_envelope: Option<bool>,
}

pub fn handle_presets_request() -> String {
    let presets = skynet_node::list_presets();
    serde_json::to_string(&presets).unwrap_or_else(|_| "[]".to_string())
}

pub fn handle_preprocess_request(body: &str) -> String {
    let req: PreprocessApiReq = serde_json::from_str(body).unwrap_or_default();

    let preset_id = req.preset.as_deref().unwrap_or("text-audit");
    let mut opts = skynet_node::default_options_for_preset(preset_id);
    if let Some(v) = req.redact_pii { opts.redact_pii = v; }
    if let Some(v) = req.normalize_minmax { opts.csv_normalize_minmax = v; }
    if let Some(v) = req.standardize_e164 { opts.standardize_e164 = v; }
    if let Some(v) = req.encrypt_envelope { opts.encrypt_envelope = v; }

    let input_text = req.input.unwrap_or_else(|| "Sample input".to_string());
    match skynet_node::run_preprocessing(&input_text, &opts) {
        Ok(res) => serde_json::to_string(&res).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => format!("{{\"error\":\"{e}\"}}"),
    }
}

pub fn extract_http_body(req: &str) -> &str {
    if let Some(pos) = req.find("\r\n\r\n") {
        &req[pos + 4..]
    } else if let Some(pos) = req.find("\n\n") {
        &req[pos + 2..]
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_presets_request() {
        let json = handle_presets_request();
        assert!(json.contains("text-audit"));
        assert!(json.contains("csv-clean-normalize"));
    }

    #[test]
    fn test_handle_preprocess_request() {
        let body = r#"{"preset":"text-audit","input":"Lien he 0912345678"}"#;
        let res = handle_preprocess_request(body);
        assert!(res.contains("[REDACTED_PHONE]"));
    }
}
