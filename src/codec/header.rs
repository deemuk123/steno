use sha2::{Sha256, Digest};
use crate::dictionary::DictionarySet;

pub const HEADER_PREFIX: &str = "[steno:v1:dict=";
pub const HEADER_SUFFIX: &str = "]";

/// Compute a short hash of the dictionary state for the header.
pub fn dict_hash(dict: &DictionarySet) -> String {
    let mut hasher = Sha256::new();
    // Sort entries for deterministic hashing
    let mut entries: Vec<(&String, &String)> = dict.entries.iter().collect();
    entries.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in entries {
        hasher.update(k.as_bytes());
        hasher.update(b"=");
        hasher.update(v.as_bytes());
        hasher.update(b";");
    }
    let result = hasher.finalize();
    hex::encode(&result[..4]) // 8 hex chars — enough to detect mismatches
}

/// Wrap compressed text with a steno header.
pub fn add_header(text: &str, dict: &DictionarySet) -> String {
    let hash = dict_hash(dict);
    format!("{}{}{}\n{}", HEADER_PREFIX, hash, HEADER_SUFFIX, text)
}

/// Check if text has a steno header. Returns the hash if present.
pub fn parse_header(text: &str) -> Option<&str> {
    let first_line = text.lines().next()?;
    if first_line.starts_with(HEADER_PREFIX) && first_line.ends_with(HEADER_SUFFIX) {
        let start = HEADER_PREFIX.len();
        let end = first_line.len() - HEADER_SUFFIX.len();
        Some(&first_line[start..end])
    } else {
        None
    }
}

/// Strip the header from compressed text, returning the body.
pub fn strip_header(text: &str) -> &str {
    if parse_header(text).is_some() {
        // Skip first line + newline
        let newline_pos = text.find('\n').unwrap_or(text.len());
        &text[newline_pos.saturating_add(1)..]
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::load_core;

    #[test]
    fn test_header_round_trip() {
        let dict = load_core();
        let text = "some compressed text here";
        let with_header = add_header(text, &dict);
        assert!(parse_header(&with_header).is_some());
        assert_eq!(strip_header(&with_header), text);
    }

    #[test]
    fn test_no_header_returns_none() {
        assert!(parse_header("plain text no header").is_none());
    }

    #[test]
    fn test_dict_hash_is_deterministic() {
        let dict = load_core();
        assert_eq!(dict_hash(&dict), dict_hash(&dict));
    }

    #[test]
    fn test_already_compressed_detection() {
        let dict = load_core();
        let compressed = add_header("text", &dict);
        assert!(parse_header(&compressed).is_some(), "should detect already-compressed text");
    }
}
