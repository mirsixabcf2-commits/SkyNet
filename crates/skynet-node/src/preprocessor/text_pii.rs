//! PII redaction and sliding-window text chunking for private document processing.

/// Redacts sensitive PII entities (phones, emails, citizen IDs, credit cards, API keys) from text.
pub fn redact_pii(input: &str) -> String {
    let mut words: Vec<String> = Vec::new();

    for token in input.split_whitespace() {
        let (prefix, core, suffix) = split_punctuation_affixes(token);
        if core.is_empty() {
            words.push(token.to_string());
            continue;
        }

        let redacted_core = sanitize_token(core);
        words.push(format!("{prefix}{redacted_core}{suffix}"));
    }

    words.join(" ")
}

/// Splits leading and trailing punctuation from an alphanumeric token core.
fn split_punctuation_affixes(token: &str) -> (&str, &str, &str) {
    let chars: Vec<char> = token.chars().collect();
    if chars.is_empty() {
        return ("", "", "");
    }

    let mut start = 0;
    while start < chars.len() && is_boundary_punct(chars[start]) {
        start += 1;
    }

    let mut end = chars.len();
    while end > start && is_boundary_punct(chars[end - 1]) {
        end -= 1;
    }

    let prefix_len: usize = chars[..start].iter().map(|c| c.len_utf8()).sum();
    let core_len: usize = chars[start..end].iter().map(|c| c.len_utf8()).sum();

    let prefix = &token[..prefix_len];
    let core = &token[prefix_len..prefix_len + core_len];
    let suffix = &token[prefix_len + core_len..];

    (prefix, core, suffix)
}

fn is_boundary_punct(c: char) -> bool {
    matches!(c, '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | '"' | '\'' | ',' | '.' | ';' | ':' | '!' | '?')
}

/// Evaluates and masks a single core token against known PII patterns.
fn sanitize_token(token: &str) -> String {
    if is_api_key(token) {
        return "[REDACTED_KEY]".to_string();
    }
    if is_email(token) {
        return "[REDACTED_EMAIL]".to_string();
    }
    if is_credit_card(token) {
        return "[REDACTED_CARD]".to_string();
    }
    if is_citizen_id(token) {
        return "[REDACTED_ID]".to_string();
    }
    if is_phone_number(token) {
        return "[REDACTED_PHONE]".to_string();
    }
    token.to_string()
}

fn is_api_key(t: &str) -> bool {
    t.starts_with("sk_live_")
        || t.starts_with("sk_test_")
        || t.starts_with("AIzaSy")
        || (t.starts_with("Bearer") && t.len() > 10)
}

fn is_email(t: &str) -> bool {
    let parts: Vec<&str> = t.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (user, domain) = (parts[0], parts[1]);
    !user.is_empty() && domain.contains('.') && domain.len() >= 3
}

fn is_credit_card(t: &str) -> bool {
    let digits: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 16 {
        return false;
    }
    t.chars().all(|c| c.is_ascii_digit() || c == '-' || c == ' ')
}

fn is_citizen_id(t: &str) -> bool {
    let is_pure_digits = t.chars().all(|c| c.is_ascii_digit());
    is_pure_digits && (t.len() == 9 || t.len() == 12)
}

fn is_phone_number(t: &str) -> bool {
    let raw: String = t.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect();
    if let Some(digits) = raw.strip_prefix("+84") {
        return digits.len() == 9 || digits.len() == 10;
    }
    if let Some(digits) = raw.strip_prefix('0') {
        return digits.len() == 9 || digits.len() == 10;
    }
    false
}

/// Splits text into overlapping sliding-window chunks.
pub fn sliding_window_chunks(text: &str, window_size: usize, overlap: usize) -> Vec<String> {
    if text.is_empty() || window_size == 0 {
        return Vec::new();
    }
    let step = if window_size > overlap {
        window_size - overlap
    } else {
        1
    };

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }
    if words.len() <= window_size {
        return vec![words.join(" ")];
    }

    let mut chunks = Vec::new();
    let mut start = 0;
    while start < words.len() {
        let end = (start + window_size).min(words.len());
        chunks.push(words[start..end].join(" "));
        if end == words.len() {
            break;
        }
        start += step;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pii_redaction() {
        let raw = "Lien he anh Nam 0912345678, email nam@domain.vn, cccd 001201012345, key sk_live_abc123456789xyz.";
        let redacted = redact_pii(raw);
        assert!(redacted.contains("[REDACTED_PHONE]"));
        assert!(redacted.contains("[REDACTED_EMAIL]"));
        assert!(redacted.contains("[REDACTED_ID]"));
        assert!(redacted.contains("[REDACTED_KEY]"));
        assert!(!redacted.contains("0912345678"));
        assert!(!redacted.contains("nam@domain.vn"));
    }

    #[test]
    fn test_sliding_window_chunks() {
        let sample = "one two three four five six seven eight nine ten";
        let chunks = sliding_window_chunks(sample, 4, 2);
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0], "one two three four");
        assert_eq!(chunks[1], "three four five six");
    }
}
