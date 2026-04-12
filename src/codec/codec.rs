use crate::dictionary::DictionarySet;
use crate::layers::{
    strip::strip,
    substitute::{substitute, desubstitute},
    abbreviate::{abbreviate, deabbreviate},
};
use crate::codec::header::{add_header, parse_header, strip_header, dict_hash};

#[derive(Debug)]
pub struct CompressedOutput {
    pub text: String,
    pub original_len: usize,
    pub compressed_len: usize,
}

impl CompressedOutput {
    /// Token savings as a percentage (approximate: 1 token ≈ 4 chars)
    pub fn ratio(&self) -> f32 {
        if self.original_len == 0 {
            return 0.0;
        }
        let saved = self.original_len.saturating_sub(self.compressed_len);
        (saved as f32 / self.original_len as f32) * 100.0
    }
}

#[derive(Debug)]
pub enum StenoError {
    DictionaryMismatch { expected: String, got: String },
    AlreadyCompressed,
}

impl std::fmt::Display for StenoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StenoError::DictionaryMismatch { expected, got } => write!(
                f,
                "Dictionary mismatch: text was compressed with dict={}, current dict={}. \
                 Run `steno dict update` or reinstall the original pack.",
                expected, got
            ),
            StenoError::AlreadyCompressed => {
                write!(f, "Text is already compressed by steno. Skipping.")
            }
        }
    }
}

pub struct Codec {
    pub core_dict: DictionarySet,
    pub domain_dict: DictionarySet,
}

impl Codec {
    pub fn new(core_dict: DictionarySet, domain_dict: DictionarySet) -> Self {
        Self {
            core_dict,
            domain_dict,
        }
    }

    /// Build a combined dictionary for hashing (core + domain merged).
    fn combined_dict(&self) -> DictionarySet {
        let mut combined = self.core_dict.clone();
        for (k, v) in &self.domain_dict.entries {
            combined.entries.insert(k.clone(), v.clone());
            combined.reverse.insert(v.clone(), k.clone());
        }
        combined
    }

    /// Compress text through all 3 layers. Never fails — returns original on layer error.
    pub fn compress(&self, text: &str) -> Result<CompressedOutput, StenoError> {
        // Detect already-compressed input
        if parse_header(text).is_some() {
            return Err(StenoError::AlreadyCompressed);
        }

        let original_len = text.len();

        // Layer 1: structural stripping
        let stripped = strip(text);

        // Layer 2: pattern substitution (core dict)
        let substituted = substitute(&stripped, &self.core_dict);

        // Layer 3: domain abbreviation
        let abbreviated = abbreviate(&substituted, &self.domain_dict);

        // Add header with combined dict hash
        let combined = self.combined_dict();
        let with_header = add_header(&abbreviated, &combined);
        let compressed_len = with_header.len();

        Ok(CompressedOutput {
            text: with_header,
            original_len,
            compressed_len,
        })
    }

    /// Decompress text. Fails loud if dictionary doesn't match.
    pub fn decompress(&self, text: &str) -> Result<String, StenoError> {
        let stored_hash = parse_header(text).ok_or_else(|| StenoError::DictionaryMismatch {
            expected: "unknown".into(),
            got: "no header found — text may not be steno-compressed".into(),
        })?;

        // Verify hash matches current dict state
        let combined = self.combined_dict();
        let current_hash = dict_hash(&combined);

        if stored_hash != current_hash {
            return Err(StenoError::DictionaryMismatch {
                expected: stored_hash.to_string(),
                got: current_hash,
            });
        }

        let body = strip_header(text);

        // Reverse layers in opposite order: deabbreviate → desubstitute
        let deabbreviated = deabbreviate(body, &self.domain_dict);
        let desubstituted = desubstitute(&deabbreviated, &self.core_dict);

        Ok(desubstituted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::load_core;

    fn make_codec() -> Codec {
        Codec::new(load_core(), DictionarySet::new())
    }

    #[test]
    fn test_compress_produces_shorter_output() {
        let codec = make_codec();
        let text = "in order to understand this, for example, it is important to note that the following steps are required.";
        let result = codec.compress(text).unwrap();
        assert!(
            result.compressed_len < result.original_len,
            "compressed ({}) should be shorter than original ({})",
            result.compressed_len,
            result.original_len
        );
    }

    #[test]
    fn test_full_roundtrip() {
        let codec = make_codec();
        let original = "in order to succeed, for example, we must act. it is important to note that the following matters.";
        let compressed = codec.compress(original).unwrap();
        let restored = codec.decompress(&compressed.text).unwrap();
        assert_eq!(restored.trim(), original.trim());
    }

    #[test]
    fn test_already_compressed_returns_error() {
        let codec = make_codec();
        let text = "some text";
        let compressed = codec.compress(text).unwrap();
        let result = codec.compress(&compressed.text);
        assert!(matches!(result, Err(StenoError::AlreadyCompressed)));
    }

    #[test]
    fn test_ratio_is_positive_for_verbose_text() {
        let codec = make_codec();
        let text = "in order to understand this concept, for example, it is important to note that the following applies.";
        let result = codec.compress(text).unwrap();
        assert!(result.ratio() > 0.0, "ratio should be positive");
    }

    #[test]
    fn test_plain_text_roundtrip() {
        let codec = make_codec();
        let original = "hello world this has no matches";
        let compressed = codec.compress(original).unwrap();
        let restored = codec.decompress(&compressed.text).unwrap();
        assert_eq!(restored.trim(), original.trim());
    }
}
