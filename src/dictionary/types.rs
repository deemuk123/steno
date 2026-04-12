use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct DictionaryMeta {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub language: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DictionaryFile {
    pub meta: DictionaryMeta,
    pub entries: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct DictionarySet {
    /// All entries merged from all loaded dictionaries
    pub entries: HashMap<String, String>,
    /// Reverse map: short_code → original phrase (for decompression)
    pub reverse: HashMap<String, String>,
}

impl DictionarySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, dict: DictionaryFile) {
        for (pattern, code) in dict.entries {
            self.reverse.insert(code.clone(), pattern.clone());
            self.entries.insert(pattern, code);
        }
    }

    pub fn compress_lookup(&self, phrase: &str) -> Option<&str> {
        self.entries.get(phrase).map(|s| s.as_str())
    }

    pub fn decompress_lookup(&self, code: &str) -> Option<&str> {
        self.reverse.get(code).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dict() -> DictionarySet {
        let mut d = DictionarySet::new();
        let file = DictionaryFile {
            meta: DictionaryMeta {
                name: "test".into(),
                description: "test dict".into(),
                author: "test".into(),
                version: "0.1.0".into(),
                language: "en".into(),
            },
            entries: {
                let mut m = HashMap::new();
                m.insert("in order to".into(), "→".into());
                m.insert("for example".into(), "e.g.".into());
                m
            },
        };
        d.merge(file);
        d
    }

    #[test]
    fn test_compress_lookup() {
        let d = make_dict();
        assert_eq!(d.compress_lookup("in order to"), Some("→"));
        assert_eq!(d.compress_lookup("unknown phrase"), None);
    }

    #[test]
    fn test_decompress_lookup() {
        let d = make_dict();
        assert_eq!(d.decompress_lookup("→"), Some("in order to"));
        assert_eq!(d.decompress_lookup("??"), None);
    }

    #[test]
    fn test_roundtrip_lookup() {
        let d = make_dict();
        let original = "in order to";
        let code = d.compress_lookup(original).unwrap();
        let restored = d.decompress_lookup(code).unwrap();
        assert_eq!(restored, original);
    }
}
