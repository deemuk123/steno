use crate::dictionary::DictionarySet;
use crate::layers::substitute::{substitute, desubstitute};

/// Layer 3: Apply domain-specific and personal abbreviations.
/// Structurally identical to Layer 2 but operates on a separate dictionary set,
/// allowing domain packs and personal dicts to be managed independently.
pub fn abbreviate(text: &str, domain_dict: &DictionarySet) -> String {
    substitute(text, domain_dict)
}

pub fn deabbreviate(text: &str, domain_dict: &DictionarySet) -> String {
    desubstitute(text, domain_dict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::types::{DictionaryFile, DictionaryMeta, DictionarySet};
    use std::collections::HashMap;

    fn make_domain_dict() -> DictionarySet {
        let mut d = DictionarySet::new();
        d.merge(DictionaryFile {
            meta: DictionaryMeta {
                name: "test-domain".into(),
                description: "test".into(),
                author: "test".into(),
                version: "0.1.0".into(),
                language: "en".into(),
            },
            entries: {
                let mut m = HashMap::new();
                m.insert("machine learning".into(), "ML".into());
                m.insert("artificial intelligence".into(), "AI".into());
                m.insert("large language model".into(), "LLM".into());
                m
            },
        });
        d
    }

    #[test]
    fn test_abbreviates_domain_term() {
        let dict = make_domain_dict();
        let result = abbreviate("machine learning is useful", &dict);
        assert!(result.contains("ML"), "expected ML but got: {}", result);
    }

    #[test]
    fn test_roundtrip() {
        let dict = make_domain_dict();
        let original = "large language model and artificial intelligence";
        let compressed = abbreviate(original, &dict);
        let restored = deabbreviate(&compressed, &dict);
        assert_eq!(restored, original);
    }

    #[test]
    fn test_empty_domain_passthrough() {
        let dict = DictionarySet::new();
        let input = "machine learning test";
        assert_eq!(abbreviate(input, &dict), input);
    }
}
