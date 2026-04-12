use super::types::{DictionaryFile, DictionarySet};

const CORE_TOML: &str = include_str!("../../dictionaries/core/universal.toml");

pub fn load_core() -> DictionarySet {
    let dict_file: DictionaryFile = toml::from_str(CORE_TOML)
        .expect("bundled universal.toml is invalid — this is a bug");
    let mut set = DictionarySet::new();
    set.merge(dict_file);
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_loads() {
        let dict = load_core();
        assert!(!dict.entries.is_empty(), "core dictionary must not be empty");
    }

    #[test]
    fn test_core_has_expected_entries() {
        let dict = load_core();
        assert_eq!(dict.compress_lookup("in order to"), Some("→"));
        assert_eq!(dict.compress_lookup("for example"), Some("e.g."));
    }

    #[test]
    fn test_core_roundtrips() {
        let dict = load_core();
        for (phrase, code) in &dict.entries {
            let restored = dict.decompress_lookup(code)
                .unwrap_or_else(|| panic!("no reverse entry for code: {}", code));
            assert_eq!(restored, phrase, "round-trip failed for: {}", phrase);
        }
    }
}
