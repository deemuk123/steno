# Steno Phase 1 — Core Rust Crate Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the core steno Rust crate — a working compress/decompress pipeline with all 3 layers and a bundled universal dictionary.

**Architecture:** A `Codec` struct orchestrates three sequential layers (strip → substitute → abbreviate). Each layer is a pure function over a string. The `DictionarySet` loads TOML entries from the bundled core dictionary plus optional user extensions. Every compressed output carries a header with a dictionary hash for guaranteed round-trip fidelity.

**Tech Stack:** Rust, `cargo`, `toml` crate (TOML parsing), `serde` + `serde_derive` (deserialization), `sha2` (dictionary hashing), `std` only otherwise.

---

### Task 1: Initialize Rust Project

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`

**Step 1: Initialize cargo project**

```bash
cd D:/Claude/steno
cargo init --name steno
```

Expected: `Cargo.toml` and `src/main.rs` created.

**Step 2: Add dependencies to `Cargo.toml`**

Replace the `[dependencies]` section:

```toml
[package]
name = "steno"
version = "0.1.0"
edition = "2021"
description = "Compress anything going into your LLM. Less tokens, same meaning."
license = "MIT"
repository = "https://github.com/deemuk123/steno"

[dependencies]
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
sha2 = "0.10"
hex = "0.4"

[lib]
name = "steno"
path = "src/lib.rs"

[[bin]]
name = "steno"
path = "src/main.rs"
```

**Step 3: Create empty `src/lib.rs`**

```rust
// steno — compress anything going into your LLM
pub mod codec;
pub mod dictionary;
pub mod layers;
```

**Step 4: Verify project compiles**

```bash
cargo build
```
Expected: compiles with warnings about empty modules (that's fine).

**Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/main.rs
git commit -m "feat: initialize steno Rust project"
```

---

### Task 2: Dictionary Types

**Files:**
- Create: `src/dictionary/mod.rs`
- Create: `src/dictionary/types.rs`
- Test: inside `src/dictionary/types.rs` under `#[cfg(test)]`

**Step 1: Write failing test**

Create `src/dictionary/types.rs`:

```rust
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
```

**Step 2: Create `src/dictionary/mod.rs`**

```rust
pub mod types;
pub use types::DictionarySet;
```

**Step 3: Update `src/lib.rs`**

```rust
pub mod codec;
pub mod dictionary;
pub mod layers;

pub use dictionary::DictionarySet;
```

**Step 4: Run tests to verify they pass**

```bash
cargo test dictionary
```
Expected: 3 tests pass.

**Step 5: Commit**

```bash
git add src/dictionary/
git commit -m "feat: add DictionarySet types with compress/decompress lookup"
```

---

### Task 3: Bundled Universal Dictionary

**Files:**
- Create: `dictionaries/core/universal.toml`
- Create: `src/dictionary/core.rs`
- Test: inside `src/dictionary/core.rs` under `#[cfg(test)]`

**Step 1: Create `dictionaries/core/universal.toml`**

```toml
[meta]
name        = "steno-core"
description = "Universal LLM pattern dictionary — works across all domains"
author      = "steno-maintainers"
version     = "0.1.0"
language    = "en"

[entries]
"in order to"                        = "→"
"it is important to note that"       = "‼"
"as mentioned above"                 = "↑ref"
"the following"                      = "ff:"
"for example"                        = "e.g."
"in other words"                     = "i.e."
"with respect to"                    = "re:"
"it should be noted"                 = "note:"
"due to the fact that"               = "because"
"in the context of"                  = "ctx:"
"as a result of"                     = "∴"
"on the other hand"                  = "↔"
"in addition to"                     = "+also"
"at the same time"                   = "meanwhile"
"in conclusion"                      = "∎"
"it is worth noting"                 = "noteworthy:"
"the purpose of"                     = "purpose:"
"in this case"                       = "here:"
"more specifically"                  = "spec:"
"in summary"                         = "∑:"
```

**Step 2: Create `src/dictionary/core.rs`**

Embed the TOML at compile time using `include_str!`:

```rust
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
```

**Step 3: Update `src/dictionary/mod.rs`**

```rust
pub mod core;
pub mod types;
pub use types::DictionarySet;
pub use core::load_core;
```

**Step 4: Run tests**

```bash
cargo test dictionary
```
Expected: 6 tests pass (3 from Task 2 + 3 new).

**Step 5: Commit**

```bash
git add dictionaries/ src/dictionary/core.rs src/dictionary/mod.rs
git commit -m "feat: bundle universal core dictionary with compile-time embed"
```

---

### Task 4: Layer 1 — Structural Stripping

**Files:**
- Create: `src/layers/mod.rs`
- Create: `src/layers/strip.rs`
- Test: inside `src/layers/strip.rs` under `#[cfg(test)]`

**Step 1: Write failing test**

Create `src/layers/strip.rs`:

```rust
/// Layer 1: Remove structural noise from text before substitution.
/// Targets: redundant blank lines, trailing whitespace, markdown decoration noise.
/// Does NOT remove meaningful markdown (headers, bullets stay — they carry structure).
pub fn strip(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::with_capacity(lines.len());
    let mut prev_blank = false;

    for line in &lines {
        let trimmed = line.trim_end();

        // Collapse multiple blank lines into one
        if trimmed.is_empty() {
            if !prev_blank {
                out.push("");
            }
            prev_blank = true;
        } else {
            out.push(trimmed);
            prev_blank = false;
        }
    }

    // Remove leading/trailing blank lines
    while out.first() == Some(&"") {
        out.remove(0);
    }
    while out.last() == Some(&"") {
        out.pop();
    }

    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strips_trailing_whitespace() {
        assert_eq!(strip("hello   \nworld  "), "hello\nworld");
    }

    #[test]
    fn test_collapses_blank_lines() {
        assert_eq!(strip("a\n\n\n\nb"), "a\n\nb");
    }

    #[test]
    fn test_removes_leading_trailing_blanks() {
        assert_eq!(strip("\n\nhello\n\n"), "hello");
    }

    #[test]
    fn test_preserves_single_blank_line() {
        assert_eq!(strip("a\n\nb"), "a\n\nb");
    }

    #[test]
    fn test_preserves_markdown_headers() {
        let input = "# Title\n\n## Section\n\nText";
        assert_eq!(strip(input), "# Title\n\n## Section\n\nText");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(strip(""), "");
    }
}
```

**Step 2: Create `src/layers/mod.rs`**

```rust
pub mod strip;
pub mod substitute;
pub mod abbreviate;
```

**Step 3: Run tests**

```bash
cargo test layers::strip
```
Expected: 6 tests pass.

**Step 4: Commit**

```bash
git add src/layers/
git commit -m "feat: add Layer 1 structural stripping"
```

---

### Task 5: Layer 2 — Pattern Substitution

**Files:**
- Create: `src/layers/substitute.rs`
- Test: inside `src/layers/substitute.rs` under `#[cfg(test)]`

**Step 1: Write the file**

```rust
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
```

**Step 2: Run tests**

```bash
cargo test layers::substitute
```
Expected: 4 tests pass.

**Step 3: Commit**

```bash
git add src/layers/substitute.rs
git commit -m "feat: add Layer 2 pattern substitution with longest-match"
```

---

### Task 6: Layer 3 — Domain Abbreviation

**Files:**
- Create: `src/layers/abbreviate.rs`
- Test: inside `src/layers/abbreviate.rs` under `#[cfg(test)]`

**Step 1: Write the file**

Layer 3 is structurally identical to Layer 2 but operates on the user/domain dictionary. By keeping them separate, each layer can evolve independently.

```rust
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
```

**Step 2: Run tests**

```bash
cargo test layers::abbreviate
```
Expected: 3 tests pass.

**Step 3: Commit**

```bash
git add src/layers/abbreviate.rs
git commit -m "feat: add Layer 3 domain abbreviation"
```

---

### Task 7: Compression Header

**Files:**
- Create: `src/codec/header.rs`
- Create: `src/codec/mod.rs`
- Test: inside `src/codec/header.rs` under `#[cfg(test)]`

**Step 1: Write `src/codec/header.rs`**

```rust
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
```

**Step 2: Create `src/codec/mod.rs`** (stub — full codec in next task)

```rust
pub mod header;
```

**Step 3: Update `src/lib.rs`**

```rust
pub mod codec;
pub mod dictionary;
pub mod layers;

pub use dictionary::DictionarySet;
```

**Step 4: Run tests**

```bash
cargo test codec::header
```
Expected: 4 tests pass.

**Step 5: Commit**

```bash
git add src/codec/
git commit -m "feat: add compression header with dictionary hash"
```

---

### Task 8: Codec — Full Compress/Decompress Pipeline

**Files:**
- Create: `src/codec/codec.rs`
- Modify: `src/codec/mod.rs`
- Test: inside `src/codec/codec.rs` under `#[cfg(test)]`

**Step 1: Write `src/codec/codec.rs`**

```rust
use crate::dictionary::DictionarySet;
use crate::layers::{strip::strip, substitute::{substitute, desubstitute}, abbreviate::{abbreviate, deabbreviate}};
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
        if self.original_len == 0 { return 0.0; }
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
            StenoError::DictionaryMismatch { expected, got } =>
                write!(f, "Dictionary mismatch: text was compressed with dict={}, current dict={}. Run `steno dict update` or reinstall the original pack.", expected, got),
            StenoError::AlreadyCompressed =>
                write!(f, "Text is already compressed by steno. Skipping."),
        }
    }
}

pub struct Codec {
    pub core_dict: DictionarySet,
    pub domain_dict: DictionarySet,
}

impl Codec {
    pub fn new(core_dict: DictionarySet, domain_dict: DictionarySet) -> Self {
        Self { core_dict, domain_dict }
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

        // Add header
        // Combine both dicts for hashing
        let mut combined = self.core_dict.clone();
        for (k, v) in &self.domain_dict.entries {
            combined.entries.insert(k.clone(), v.clone());
            combined.reverse.insert(v.clone(), k.clone());
        }
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

        // Verify hash
        let mut combined = self.core_dict.clone();
        for (k, v) in &self.domain_dict.entries {
            combined.entries.insert(k.clone(), v.clone());
            combined.reverse.insert(v.clone(), k.clone());
        }
        let current_hash = dict_hash(&combined);

        if stored_hash != current_hash {
            return Err(StenoError::DictionaryMismatch {
                expected: stored_hash.to_string(),
                got: current_hash,
            });
        }

        let body = strip_header(text);

        // Reverse layers in opposite order
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
        assert!(result.compressed_len < result.original_len,
            "compressed ({}) should be shorter than original ({})",
            result.compressed_len, result.original_len);
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
```

**Step 2: Update `src/codec/mod.rs`**

```rust
pub mod codec;
pub mod header;
pub use codec::{Codec, CompressedOutput, StenoError};
```

**Step 3: Update `src/lib.rs`**

```rust
pub mod codec;
pub mod dictionary;
pub mod layers;

pub use codec::{Codec, CompressedOutput, StenoError};
pub use dictionary::DictionarySet;
```

**Step 4: Run all tests**

```bash
cargo test
```
Expected: all tests pass (17+ tests across all modules).

**Step 5: Commit**

```bash
git add src/codec/codec.rs src/codec/mod.rs src/lib.rs
git commit -m "feat: complete compress/decompress pipeline — Phase 1 core crate done"
```

---

### Task 9: Update README and Project State

**Files:**
- Modify: `README.md`
- Modify: `docs/project-state.md`

**Step 1: Update README status badge**

Change the Status section from:
```
> 🔵 **In design**
```
to:
```
> 🟡 **Phase 1 complete** — Core crate built. CLI coming in Phase 2.
```

Add to the Journey Log:
```
| 2026-04-12 | Phase 1 complete — core Rust crate with 3-layer pipeline, universal dictionary, full test coverage |
```

**Step 2: Update `docs/project-state.md`**

Change Current Phase to:
```
**Phase 2 — CLI** (next)
Phase 1 core crate is complete. All tests passing.
Next: build the CLI (steno compress, decompress, stats, dict, learn, serve).
```

**Step 3: Commit**

```bash
git add README.md docs/project-state.md
git commit -m "docs: mark Phase 1 complete, update project state"
```
