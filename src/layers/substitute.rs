use crate::dictionary::DictionarySet;

/// Layer 2: Replace verbose phrases with short codes from the dictionary.
/// Matches are case-insensitive, whole-phrase only (not mid-word).
/// Applies longest-match first to avoid partial replacements.
pub fn substitute(text: &str, dict: &DictionarySet) -> String {
    if dict.entries.is_empty() {
        return text.to_string();
    }

    // Sort patterns longest-first to prefer longer matches
    let mut patterns: Vec<(&str, &str)> = dict.entries
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    patterns.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let lower = text.to_lowercase();
    let mut result = text.to_string();
    let mut lower_result = lower.clone();

    for (pattern, code) in &patterns {
        let pat_lower = pattern.to_lowercase();
        // Replace all occurrences, preserving surrounding context
        while let Some(pos) = lower_result.find(pat_lower.as_str()) {
            // Only replace if at word boundary (preceded/followed by space, newline, or start/end)
            let before_ok = pos == 0 || {
                let c = lower_result.chars().nth(pos - 1).unwrap_or(' ');
                c == ' ' || c == '\n' || c == '\t'
            };
            let after_pos = pos + pat_lower.len();
            let after_ok = after_pos >= lower_result.len() || {
                let c = lower_result.chars().nth(after_pos).unwrap_or(' ');
                c == ' ' || c == '\n' || c == '\t' || c == '.' || c == ',' || c == ':'
            };

            if before_ok && after_ok {
                result.replace_range(pos..pos + pattern.len(), code);
                lower_result.replace_range(pos..pos + pat_lower.len(), code);
            } else {
                break; // avoid infinite loop on non-boundary match
            }
        }
    }
    result
}

/// Reverse substitution: replace short codes back with original phrases.
pub fn desubstitute(text: &str, dict: &DictionarySet) -> String {
    if dict.reverse.is_empty() {
        return text.to_string();
    }

    let mut result = text.to_string();
    // Sort codes longest-first to avoid partial replacements
    let mut codes: Vec<(&str, &str)> = dict.reverse
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    codes.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (code, phrase) in codes {
        result = result.replace(code, phrase);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::load_core;

    #[test]
    fn test_substitutes_known_phrase() {
        let dict = load_core();
        let result = substitute("in order to succeed", &dict);
        assert!(result.contains("→"), "expected → but got: {}", result);
    }

    #[test]
    fn test_no_change_on_unknown() {
        let dict = load_core();
        let input = "completely unknown phrase xyz";
        assert_eq!(substitute(input, &dict), input);
    }

    #[test]
    fn test_roundtrip() {
        let dict = load_core();
        let original = "for example this is in order to demonstrate";
        let compressed = substitute(original, &dict);
        let restored = desubstitute(&compressed, &dict);
        assert_eq!(restored, original);
    }

    #[test]
    fn test_empty_dict_passthrough() {
        let dict = DictionarySet::new();
        let input = "in order to test";
        assert_eq!(substitute(input, &dict), input);
    }
}
