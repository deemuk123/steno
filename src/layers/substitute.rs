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

    let mut result = text.to_string();

    for (pattern, code) in &patterns {
        let pat_lower = pattern.to_lowercase();
        // Recompute lowercase view after each pattern (result may have changed)
        let lower_result = result.to_lowercase();
        let mut output = String::with_capacity(result.len());
        let mut last_end = 0usize; // byte offset — flushed up to here
        let mut search_start = 0usize; // byte offset — search from here

        loop {
            let Some(rel_pos) = lower_result[search_start..].find(pat_lower.as_str()) else {
                break;
            };
            let pos = search_start + rel_pos;
            let end_pos = pos + pat_lower.len();

            // Byte-safe word-boundary checks (chars().nth(byte_idx) is WRONG for multi-byte Unicode)
            let before_ok = pos == 0 || {
                let c = lower_result[..pos].chars().last().unwrap_or(' ');
                c == ' ' || c == '\n' || c == '\t'
            };
            let after_ok = end_pos >= lower_result.len() || {
                let c = lower_result[end_pos..].chars().next().unwrap_or(' ');
                c == ' ' || c == '\n' || c == '\t' || c == '.' || c == ',' || c == ':'
            };

            if before_ok && after_ok {
                // Flush original bytes up to this match, then emit the replacement code
                output.push_str(&result[last_end..pos]);
                output.push_str(code);
                last_end = end_pos;
                search_start = end_pos;
            } else {
                // Non-boundary occurrence — skip past it and keep searching
                let skip = lower_result[pos..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                search_start = pos + skip;
            }
        }
        // Flush remainder
        output.push_str(&result[last_end..]);
        result = output;
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
