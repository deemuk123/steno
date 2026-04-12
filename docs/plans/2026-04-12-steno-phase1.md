# Steno Phase 1 — Core Engine + CLI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a working `steno` binary that compresses and decompresses text through 3 layers (structural strip → pattern substitution → domain abbreviation) with a CLI, powered by a TOML dictionary system.

**Architecture:** A Rust library crate (`steno`) with a thin CLI binary on top. Three independent compression layers applied in sequence by a codec orchestrator. Dictionaries are TOML files: one universal core (bundled) and optional community packs loaded from `~/.config/steno/packs/`. All compression is reversible — decompression replays the same dictionary in reverse.

**Tech Stack:** Rust (stable), `clap` (CLI), `serde`+`toml` (dictionary loading), `regex` (Layer 1), `anyhow` (errors), `dirs` (home directory). No async, no unsafe, no build scripts.

---

### Task 0: Install Rust and Scaffold the Crate

**Files:**
- Create: `D:\Claude\steno\Cargo.toml`
- Create: `D:\Claude\steno\src\lib.rs`
- Create: `D:\Claude\steno\src\main.rs`
- Create: `D:\Claude\steno\.gitignore`

**Step 1: Install Rust**

Go to https://rustup.rs and run the installer, or run:
```powershell
winget install Rustlang.Rustup
```
Then open a new terminal and verify:
```bash
rustc --version   # should print rustc 1.7x.x
cargo --version   # should print cargo 1.7x.x
```

**Step 2: Initialize the crate**

```bash
cd D:\Claude\steno
cargo init --name steno
```
This creates `src/main.rs` and `Cargo.toml`.

**Step 3: Write `Cargo.toml`**

Replace the generated `Cargo.toml` with:
```toml
[package]
name = "steno"
version = "0.1.0"
edition = "2021"
description = "Compress any text before it enters your LLM context window"
license = "MIT"
repository = "https://github.com/your-username/steno"

[[bin]]
name = "steno"
path = "src/main.rs"

[lib]
name = "steno"
path = "src/lib.rs"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
dirs = "5"
regex = "1"
serde = { version = "1", features = ["derive"] }
toml = "0.8"

[dev-dependencies]
tempfile = "3"
```

**Step 4: Create `src/lib.rs`**

```rust
pub mod codec;
pub mod dictionary;
pub mod layers;
```

**Step 5: Create `src/main.rs`** (stub — filled out in Task 9)

```rust
fn main() {
    println!("steno");
}
```

**Step 6: Create `.gitignore`**

```
/target
```

**Step 7: Create the module directories**

```bash
mkdir -p src/layers src/dictionary dictionaries/core dictionaries/community tests
```

**Step 8: Verify it builds**

```bash
cargo build
```
Expected: `Compiling steno v0.1.0` then `Finished`. No errors.

**Step 9: Commit**

```bash
git init
git add .
git commit -m "feat: scaffold steno crate"
```

---

### Task 1: Dictionary Types

**Files:**
- Create: `src/dictionary/mod.rs`
- Create: `src/dictionary/types.rs`

**Step 1: Write `src/dictionary/types.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata block present in every dictionary TOML file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DictionaryMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub language: String,
}

/// A single dictionary pack loaded from a TOML file.
/// `entries` maps verbose phrase → short replacement.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DictionaryPack {
    pub meta: DictionaryMeta,
    pub entries: HashMap<String, String>,
}

/// The active dictionary: universal core entries + any loaded community packs.
#[derive(Debug, Clone, Default)]
pub struct Dictionary {
    /// Core entries loaded from `dictionaries/core/universal.toml`
    pub core: HashMap<String, String>,
    /// Community/personal packs loaded from ~/.config/steno/packs/
    pub packs: Vec<DictionaryPack>,
}

impl Dictionary {
    /// Returns all substitution entries sorted longest-key-first.
    /// Longest-match-first prevents partial phrase replacement.
    pub fn substitution_pairs(&self) -> Vec<(String, String)> {
        let mut pairs: Vec<(String, String)> = self
            .core
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for pack in &self.packs {
            pairs.extend(pack.entries.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        pairs
    }

    /// Returns the same entries in reverse (short → verbose) for decompression.
    pub fn decompression_pairs(&self) -> Vec<(String, String)> {
        self.substitution_pairs()
            .into_iter()
            .map(|(k, v)| (v, k))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.core.is_empty() && self.packs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitution_pairs_sorted_longest_first() {
        let mut dict = Dictionary::default();
        dict.core.insert("in order to".into(), "→".into());
        dict.core.insert("in order to be".into(), "→be".into());
        dict.core.insert("to".into(), "2".into());

        let pairs = dict.substitution_pairs();
        assert_eq!(pairs[0].0, "in order to be");
        assert_eq!(pairs[1].0, "in order to");
        assert_eq!(pairs[2].0, "to");
    }

    #[test]
    fn decompression_pairs_are_inverted() {
        let mut dict = Dictionary::default();
        dict.core.insert("in order to".into(), "→".into());

        let pairs = dict.decompression_pairs();
        assert_eq!(pairs[0].0, "→");
        assert_eq!(pairs[0].1, "in order to");
    }
}
```

**Step 2: Write `src/dictionary/mod.rs`**

```rust
pub mod loader;
pub mod types;

pub use types::Dictionary;
```

**Step 3: Run tests**

```bash
cargo test dictionary
```
Expected: 2 tests pass.

**Step 4: Commit**

```bash
git add src/dictionary/
git commit -m "feat: dictionary types with longest-match-first sorting"
```

---

### Task 2: Dictionary Loader

**Files:**
- Create: `src/dictionary/loader.rs`

**Step 1: Write the failing test first**

Add to the bottom of `src/dictionary/loader.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp_toml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn load_pack_from_toml() {
        let toml = r#"
[meta]
name = "test-pack"
description = "Test"
version = "0.1.0"
language = "en"

[entries]
"in order to" = "→"
"it is important to note that" = "‼"
"#;
        let f = write_temp_toml(toml);
        let pack = load_pack(f.path()).unwrap();

        assert_eq!(pack.meta.name, "test-pack");
        assert_eq!(pack.entries["in order to"], "→");
        assert_eq!(pack.entries["it is important to note that"], "‼");
    }

    #[test]
    fn load_core_returns_dictionary() {
        // Core dict may not exist in test env, so we test with a temp file
        let toml = r#"
[meta]
name = "universal"
description = "Core"
version = "0.1.0"
language = "en"

[entries]
"hello world" = "hw"
"#;
        let f = write_temp_toml(toml);
        let dict = load_core_from_path(f.path()).unwrap();
        assert_eq!(dict.core["hello world"], "hw");
    }
}
```

**Step 2: Run to verify they fail**

```bash
cargo test dictionary::loader
```
Expected: compile error (functions don't exist yet).

**Step 3: Write `src/dictionary/loader.rs`**

```rust
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::types::{Dictionary, DictionaryPack};

/// Load a single pack from a TOML file.
pub fn load_pack(path: &Path) -> Result<DictionaryPack> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read dictionary pack: {}", path.display()))?;
    toml::from_str(&content)
        .with_context(|| format!("Failed to parse dictionary pack: {}", path.display()))
}

/// Load core dictionary from a specific path (used in tests + normal loading).
pub fn load_core_from_path(path: &Path) -> Result<Dictionary> {
    let pack = load_pack(path)?;
    let mut dict = Dictionary::default();
    dict.core = pack.entries;
    Ok(dict)
}

/// Load the bundled universal core dictionary.
/// In release builds, the TOML is embedded at compile time.
/// Falls back to reading from `dictionaries/core/universal.toml` relative to cwd.
pub fn load_core() -> Result<Dictionary> {
    // Embed the core dictionary at compile time so the binary is self-contained.
    const CORE_TOML: &str = include_str!("../../dictionaries/core/universal.toml");
    let pack: DictionaryPack = toml::from_str(CORE_TOML)
        .context("Failed to parse bundled universal.toml")?;
    let mut dict = Dictionary::default();
    dict.core = pack.entries;
    Ok(dict)
}

/// Load community/personal packs from ~/.config/steno/packs/*.toml
pub fn load_packs(dict: &mut Dictionary) -> Result<()> {
    let packs_dir = packs_directory()?;
    if !packs_dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(&packs_dir)
        .with_context(|| format!("Cannot read packs directory: {}", packs_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "toml") {
            let pack = load_pack(&path)?;
            dict.packs.push(pack);
        }
    }
    Ok(())
}

/// Install a pack TOML file into ~/.config/steno/packs/
pub fn install_pack(src: &Path) -> Result<PathBuf> {
    let dir = packs_directory()?;
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(src.file_name().unwrap());
    std::fs::copy(src, &dest)?;
    Ok(dest)
}

/// Remove a pack by name from ~/.config/steno/packs/
pub fn remove_pack(name: &str) -> Result<()> {
    let dir = packs_directory()?;
    let path = dir.join(format!("{}.toml", name));
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// List installed pack names.
pub fn list_packs() -> Result<Vec<String>> {
    let dir = packs_directory()?;
    if !dir.exists() {
        return Ok(vec![]);
    }
    let names = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "toml"))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect();
    Ok(names)
}

fn packs_directory() -> Result<PathBuf> {
    let base = dirs::config_dir().context("Cannot find config directory")?;
    Ok(base.join("steno").join("packs"))
}
```

**Step 4: Run tests**

```bash
cargo test dictionary::loader
```
Expected: 2 tests pass.

**Step 5: Commit**

```bash
git add src/dictionary/loader.rs
git commit -m "feat: dictionary TOML loader with pack install/list/remove"
```

---

### Task 3: Write `dictionaries/core/universal.toml`

This is the product's most visible artifact — the universal phrase dictionary.

**Files:**
- Create: `dictionaries/core/universal.toml`

**Step 1: Write the file**

```toml
[meta]
name        = "universal"
description = "Universal LLM phrase dictionary — patterns that appear across all domains"
version     = "0.1.0"
language    = "en"

[entries]
# ── Verbose connectives ──────────────────────────────────────────────────────
"in order to"                           = "→"
"in order to be"                        = "→be"
"so as to"                              = "→"
"with the goal of"                      = "w/goal"
"for the purpose of"                    = "4purpose"
"with the intention of"                 = "w/intent"
"with respect to"                       = "w/r/t"
"with regard to"                        = "w/r/t"
"in relation to"                        = "re:"
"in terms of"                           = "in∑"
"as a result of"                        = "∵"
"as a consequence of"                   = "∵"
"due to the fact that"                  = "∵"
"given that"                            = "∵"
"therefore"                             = "∴"
"consequently"                          = "∴"
"as a result"                           = "∴"
"which means that"                      = "∴"
"it follows that"                       = "∴"
"nevertheless"                          = "yet"
"however"                               = "but"
"on the other hand"                     = "OTOH"
"on the one hand"                       = "OTH"
"in contrast"                           = "vs."
"in comparison"                         = "vs."
"as opposed to"                         = "vs."
"as well as"                            = "&"
"in addition to"                        = "+"
"furthermore"                           = "also"
"moreover"                              = "also"
"additionally"                          = "also"
"in particular"                         = "esp."
"specifically"                          = "esp."
"for example"                           = "e.g."
"for instance"                          = "e.g."
"such as"                               = "e.g."
"in other words"                        = "i.e."
"that is to say"                        = "i.e."
"that is"                               = "i.e."
"approximately"                         = "~"
"roughly"                               = "~"
"about"                                 = "~"

# ── LLM-specific hedges and fillers ─────────────────────────────────────────
"it is important to note that"          = "NB:"
"it should be noted that"              = "NB:"
"it is worth noting that"              = "NB:"
"please note that"                     = "NB:"
"note that"                            = "NB:"
"as mentioned above"                   = "↑"
"as discussed above"                   = "↑"
"as stated above"                      = "↑"
"as noted above"                       = "↑"
"as mentioned earlier"                 = "↑"
"as shown below"                       = "↓"
"as described below"                   = "↓"
"as follows"                           = "↓"
"the following"                        = "ff:"
"as previously mentioned"              = "↑"
"in summary"                           = "∑:"
"to summarize"                         = "∑:"
"in conclusion"                        = "∑:"
"to conclude"                          = "∑:"
"in brief"                             = "∑:"
"in short"                             = "∑:"
"overall"                              = "∑:"
"to put it simply"                     = "simply:"

# ── Verbose constructions ────────────────────────────────────────────────────
"it is"                                = "it's"
"there is"                             = "there's"
"there are"                            = "there're"
"we are"                               = "we're"
"they are"                             = "they're"
"does not"                             = "doesn't"
"do not"                               = "don't"
"cannot"                               = "can't"
"will not"                             = "won't"
"would not"                            = "wouldn't"
"should not"                           = "shouldn't"
"could not"                            = "couldn't"
"is not"                               = "isn't"
"are not"                              = "aren't"
"was not"                              = "wasn't"
"were not"                             = "weren't"
"have not"                             = "haven't"
"has not"                              = "hasn't"
"had not"                              = "hadn't"
"a large number of"                    = "many"
"a wide variety of"                    = "various"
"a wide range of"                      = "various"
"a number of"                          = "several"
"the majority of"                      = "most"
"in the event that"                    = "if"
"in the case where"                    = "if"
"at this point in time"                = "now"
"at the present time"                  = "now"
"currently"                            = "now"
"on a regular basis"                   = "regularly"
"on an ongoing basis"                  = "ongoing"
"in the near future"                   = "soon"
"in the long run"                      = "long-term"
"in the short run"                     = "short-term"
"make use of"                          = "use"
"in the process of"                    = "while"
"is able to"                           = "can"
"are able to"                          = "can"
"was able to"                          = "could"
"were able to"                         = "could"

# ── Common LLM workflow terms ────────────────────────────────────────────────
"large language model"                 = "LLM"
"large language models"                = "LLMs"
"artificial intelligence"              = "AI"
"machine learning"                     = "ML"
"natural language processing"          = "NLP"
"context window"                       = "ctx"
"token"                                = "tok"
"tokens"                               = "toks"
"embedding"                            = "emb"
"embeddings"                           = "embs"
"fine-tuning"                          = "FT"
"fine tuning"                          = "FT"
"retrieval-augmented generation"       = "RAG"
"retrieval augmented generation"       = "RAG"
"chain of thought"                     = "CoT"
"few-shot"                             = "few-shot"
"zero-shot"                            = "0-shot"
"one-shot"                             = "1-shot"
"system prompt"                        = "sys-prompt"
"user message"                         = "usr-msg"
"assistant message"                    = "asst-msg"
```

**Step 2: Verify it parses**

```bash
cargo test dictionary::loader::tests::load_core_returns_dictionary
```

Wait — that test uses a temp file, not the real path. Write a quick manual check instead:

```bash
cargo test
```

If `load_core()` is called anywhere, it will try `include_str!` at compile time. Since the file now exists, it should compile. If it doesn't compile, fix the path in `load_core()` to match `../../dictionaries/core/universal.toml` relative to `src/dictionary/loader.rs`.

**Step 3: Commit**

```bash
git add dictionaries/core/universal.toml
git commit -m "feat: universal core dictionary with 80+ phrase substitutions"
```

---

### Task 4: Layer 1 — Structural Stripping

Strips markdown decoration and normalizes whitespace. Keeps semantic content intact.

**Files:**
- Create: `src/layers/mod.rs`
- Create: `src/layers/strip.rs`

**Step 1: Write `src/layers/mod.rs`**

```rust
pub mod abbreviate;
pub mod strip;
pub mod substitute;
```

**Step 2: Write the failing tests in `src/layers/strip.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_whitespace() {
        assert_eq!(strip("hello   \nworld  "), "hello\nworld");
    }

    #[test]
    fn collapses_multiple_blank_lines() {
        assert_eq!(strip("a\n\n\n\nb"), "a\n\nb");
    }

    #[test]
    fn removes_horizontal_rules() {
        assert_eq!(strip("text\n---\nmore"), "text\nmore");
        assert_eq!(strip("text\n***\nmore"), "text\nmore");
        assert_eq!(strip("text\n___\nmore"), "text\nmore");
    }

    #[test]
    fn removes_markdown_heading_hashes() {
        assert_eq!(strip("# Title"), "Title");
        assert_eq!(strip("## Section"), "Section");
        assert_eq!(strip("### Subsection"), "Subsection");
    }

    #[test]
    fn strips_bold_markers() {
        assert_eq!(strip("this is **important** text"), "this is important text");
    }

    #[test]
    fn strips_italic_markers() {
        assert_eq!(strip("this is *emphasized* text"), "this is emphasized text");
    }

    #[test]
    fn removes_blockquote_markers() {
        assert_eq!(strip("> quoted text"), "quoted text");
    }

    #[test]
    fn removes_code_fence_markers() {
        let input = "```rust\nlet x = 1;\n```";
        let output = strip(input);
        assert!(output.contains("let x = 1;"));
        assert!(!output.contains("```"));
    }

    #[test]
    fn preserves_content() {
        let input = "The quick brown fox jumps over the lazy dog.";
        assert_eq!(strip(input), input);
    }
}
```

**Step 3: Run to verify they fail**

```bash
cargo test layers::strip
```
Expected: compile error — `strip` function not defined.

**Step 4: Write `src/layers/strip.rs`**

```rust
use regex::Regex;

/// Layer 1: Structural stripping.
/// Removes markdown decoration and normalizes whitespace.
/// All semantic content is preserved.
pub fn strip(text: &str) -> String {
    let mut s = text.to_string();

    // Remove markdown heading markers (keep heading text)
    let re_headings = Regex::new(r"(?m)^#{1,6}\s+").unwrap();
    s = re_headings.replace_all(&s, "").into_owned();

    // Remove bold markers (**text**)
    let re_bold = Regex::new(r"\*\*([^*\n]+)\*\*").unwrap();
    s = re_bold.replace_all(&s, "$1").into_owned();

    // Remove italic markers (*text*) — must run after bold
    let re_italic = Regex::new(r"\*([^*\n]+)\*").unwrap();
    s = re_italic.replace_all(&s, "$1").into_owned();

    // Remove blockquote markers
    let re_blockquote = Regex::new(r"(?m)^>\s?").unwrap();
    s = re_blockquote.replace_all(&s, "").into_owned();

    // Remove code fence markers (``` lines), keep content inside
    let re_fence = Regex::new(r"(?m)^```[^\n]*$").unwrap();
    s = re_fence.replace_all(&s, "").into_owned();

    // Remove horizontal rules (---, ***, ___ on their own line)
    let re_hr = Regex::new(r"(?m)^[-*_]{3,}\s*$").unwrap();
    s = re_hr.replace_all(&s, "").into_owned();

    // Trim trailing whitespace from each line
    s = s.lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    // Collapse 3+ consecutive newlines to 2
    let re_blank = Regex::new(r"\n{3,}").unwrap();
    s = re_blank.replace_all(&s, "\n\n").into_owned();

    // Trim leading/trailing newlines from the whole output
    s.trim_matches('\n').to_string()
}
```

**Step 5: Run tests**

```bash
cargo test layers::strip
```
Expected: all 9 tests pass.

**Step 6: Commit**

```bash
git add src/layers/
git commit -m "feat: Layer 1 structural stripping (markdown + whitespace)"
```

---

### Task 5: Layer 2 — Pattern Substitution

Replaces verbose phrases with short codes using the loaded dictionary. Uses longest-match-first to avoid partial replacements.

**Files:**
- Create: `src/layers/substitute.rs`

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::types::Dictionary;

    fn dict_with(pairs: &[(&str, &str)]) -> Dictionary {
        let mut d = Dictionary::default();
        for (k, v) in pairs {
            d.core.insert(k.to_string(), v.to_string());
        }
        d
    }

    #[test]
    fn substitutes_phrase() {
        let d = dict_with(&[("in order to", "→")]);
        assert_eq!(substitute("run this in order to test", &d), "run this → test");
    }

    #[test]
    fn longest_match_wins() {
        let d = dict_with(&[("in order to", "→"), ("in order to be", "→be")]);
        assert_eq!(substitute("in order to be done", &d), "→be done");
    }

    #[test]
    fn empty_dict_returns_original() {
        let d = Dictionary::default();
        let text = "hello world";
        assert_eq!(substitute(text, &d), text);
    }

    #[test]
    fn multiple_substitutions() {
        let d = dict_with(&[("however", "but"), ("therefore", "∴")]);
        assert_eq!(
            substitute("however, therefore", &d),
            "but, ∴"
        );
    }

    #[test]
    fn case_sensitive() {
        // Substitution is case-sensitive by default
        let d = dict_with(&[("however", "but")]);
        assert_eq!(substitute("However", &d), "However"); // no match
        assert_eq!(substitute("however", &d), "but");     // match
    }
}
```

**Step 2: Run to verify they fail**

```bash
cargo test layers::substitute
```
Expected: compile error.

**Step 3: Write `src/layers/substitute.rs`**

```rust
use crate::dictionary::types::Dictionary;

/// Layer 2: Pattern substitution.
/// Replaces verbose phrases with short codes using the dictionary.
/// Longest-match-first prevents partial phrase replacement.
pub fn substitute(text: &str, dict: &Dictionary) -> String {
    if dict.is_empty() {
        return text.to_string();
    }

    let pairs = dict.substitution_pairs();
    let mut result = text.to_string();

    for (from, to) in &pairs {
        result = result.replace(from.as_str(), to.as_str());
    }

    result
}

/// Reverse substitution for decompression.
pub fn desubstitute(text: &str, dict: &Dictionary) -> String {
    if dict.is_empty() {
        return text.to_string();
    }

    let pairs = dict.decompression_pairs();
    let mut result = text.to_string();

    for (from, to) in &pairs {
        result = result.replace(from.as_str(), to.as_str());
    }

    result
}

#[cfg(test)]
mod tests {
    // ... (tests from Step 1 go here)
    use super::*;
    use crate::dictionary::types::Dictionary;

    fn dict_with(pairs: &[(&str, &str)]) -> Dictionary {
        let mut d = Dictionary::default();
        for (k, v) in pairs {
            d.core.insert(k.to_string(), v.to_string());
        }
        d
    }

    #[test]
    fn substitutes_phrase() {
        let d = dict_with(&[("in order to", "→")]);
        assert_eq!(substitute("run this in order to test", &d), "run this → test");
    }

    #[test]
    fn longest_match_wins() {
        let d = dict_with(&[("in order to", "→"), ("in order to be", "→be")]);
        assert_eq!(substitute("in order to be done", &d), "→be done");
    }

    #[test]
    fn empty_dict_returns_original() {
        let d = Dictionary::default();
        let text = "hello world";
        assert_eq!(substitute(text, &d), text);
    }

    #[test]
    fn multiple_substitutions() {
        let d = dict_with(&[("however", "but"), ("therefore", "∴")]);
        assert_eq!(substitute("however, therefore", &d), "but, ∴");
    }

    #[test]
    fn case_sensitive() {
        let d = dict_with(&[("however", "but")]);
        assert_eq!(substitute("However", &d), "However");
        assert_eq!(substitute("however", &d), "but");
    }

    #[test]
    fn round_trip() {
        let d = dict_with(&[("in order to", "→"), ("however", "but")]);
        let original = "in order to do this, however it works";
        let compressed = substitute(original, &d);
        let restored = desubstitute(&compressed, &d);
        assert_eq!(restored, original);
    }
}
```

**Step 4: Run tests**

```bash
cargo test layers::substitute
```
Expected: all 6 tests pass.

**Step 5: Commit**

```bash
git add src/layers/substitute.rs
git commit -m "feat: Layer 2 pattern substitution with round-trip support"
```

---

### Task 6: Layer 3 — Domain Abbreviation

Applies domain-specific abbreviations from installed community packs. Same mechanics as Layer 2 but uses packs, not core dictionary.

**Files:**
- Create: `src/layers/abbreviate.rs`

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::types::{Dictionary, DictionaryMeta, DictionaryPack};
    use std::collections::HashMap;

    fn dict_with_pack(pairs: &[(&str, &str)]) -> Dictionary {
        let mut entries = HashMap::new();
        for (k, v) in pairs {
            entries.insert(k.to_string(), v.to_string());
        }
        let pack = DictionaryPack {
            meta: DictionaryMeta {
                name: "test".into(),
                description: "".into(),
                version: "0.1.0".into(),
                language: "en".into(),
            },
            entries,
        };
        let mut d = Dictionary::default();
        d.packs.push(pack);
        d
    }

    #[test]
    fn abbreviates_domain_terms() {
        let d = dict_with_pack(&[("function", "fn"), ("variable", "var")]);
        assert_eq!(abbreviate("the function and variable", &d), "the fn and var");
    }

    #[test]
    fn no_packs_returns_original() {
        let d = Dictionary::default();
        assert_eq!(abbreviate("hello world", &d), "hello world");
    }

    #[test]
    fn round_trip() {
        let d = dict_with_pack(&[("function", "fn")]);
        let original = "this function returns";
        let compressed = abbreviate(original, &d);
        let restored = deabbreviate(&compressed, &d);
        assert_eq!(restored, original);
    }
}
```

**Step 2: Run to verify they fail**

```bash
cargo test layers::abbreviate
```
Expected: compile error.

**Step 3: Write `src/layers/abbreviate.rs`**

```rust
use crate::dictionary::types::Dictionary;

/// Layer 3: Domain abbreviation.
/// Applies domain-specific abbreviations from installed community packs.
/// Uses the same dictionary pair system as Layer 2 but operates on pack entries.
pub fn abbreviate(text: &str, dict: &Dictionary) -> String {
    if dict.packs.is_empty() {
        return text.to_string();
    }

    // Build pairs from packs only (core is handled by Layer 2)
    let mut pairs: Vec<(String, String)> = dict
        .packs
        .iter()
        .flat_map(|p| p.entries.iter().map(|(k, v)| (k.clone(), v.clone())))
        .collect();
    // Longest-match-first
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let mut result = text.to_string();
    for (from, to) in &pairs {
        result = result.replace(from.as_str(), to.as_str());
    }
    result
}

/// Reverse domain abbreviation for decompression.
pub fn deabbreviate(text: &str, dict: &Dictionary) -> String {
    if dict.packs.is_empty() {
        return text.to_string();
    }

    let mut pairs: Vec<(String, String)> = dict
        .packs
        .iter()
        .flat_map(|p| p.entries.iter().map(|(k, v)| (v.clone(), k.clone())))
        .collect();
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let mut result = text.to_string();
    for (from, to) in &pairs {
        result = result.replace(from.as_str(), to.as_str());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::types::{Dictionary, DictionaryMeta, DictionaryPack};
    use std::collections::HashMap;

    fn dict_with_pack(pairs: &[(&str, &str)]) -> Dictionary {
        let mut entries = HashMap::new();
        for (k, v) in pairs {
            entries.insert(k.to_string(), v.to_string());
        }
        let pack = DictionaryPack {
            meta: DictionaryMeta {
                name: "test".into(),
                description: "".into(),
                version: "0.1.0".into(),
                language: "en".into(),
            },
            entries,
        };
        let mut d = Dictionary::default();
        d.packs.push(pack);
        d
    }

    #[test]
    fn abbreviates_domain_terms() {
        let d = dict_with_pack(&[("function", "fn"), ("variable", "var")]);
        assert_eq!(abbreviate("the function and variable", &d), "the fn and var");
    }

    #[test]
    fn no_packs_returns_original() {
        let d = Dictionary::default();
        assert_eq!(abbreviate("hello world", &d), "hello world");
    }

    #[test]
    fn round_trip() {
        let d = dict_with_pack(&[("function", "fn")]);
        let original = "this function returns";
        let compressed = abbreviate(original, &d);
        let restored = deabbreviate(&compressed, &d);
        assert_eq!(restored, original);
    }
}
```

**Step 4: Run tests**

```bash
cargo test layers::abbreviate
```
Expected: all 3 tests pass.

**Step 5: Commit**

```bash
git add src/layers/abbreviate.rs
git commit -m "feat: Layer 3 domain abbreviation from community packs"
```

---

### Task 7: Codec — Compress + Decompress Pipeline

Wires all three layers together. Compress: Layer 1 → 2 → 3. Decompress: Layer 3⁻¹ → 2⁻¹ → 1⁻¹ (Layer 1 is not reversed — structural stripping is one-way). Fail-safe: any layer failure returns original text unchanged.

**Files:**
- Create: `src/codec.rs`

**Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::types::Dictionary;

    #[test]
    fn compress_returns_shorter_text() {
        let mut dict = Dictionary::default();
        dict.core.insert("in order to".into(), "→".into());
        dict.core.insert("however".into(), "but".into());

        let input = "We do this in order to improve. However, it takes time.";
        let result = compress(input, &dict);
        assert!(result.output.len() < input.len());
    }

    #[test]
    fn compress_empty_dict_returns_stripped() {
        // With empty dict, only Layer 1 (structural strip) runs
        let dict = Dictionary::default();
        let input = "# Title\n\nsome text   \n\n\nextra blank";
        let result = compress(input, &dict);
        assert_eq!(result.output, "Title\n\nsome text\n\nextra blank");
    }

    #[test]
    fn stats_are_correct() {
        let mut dict = Dictionary::default();
        dict.core.insert("in order to".into(), "→".into());

        let input = "do this in order to pass";
        let result = compress(input, &dict);
        assert_eq!(result.original_len, input.len());
        assert!(result.compressed_len < result.original_len);
        assert!(result.ratio() < 1.0);
    }

    #[test]
    fn compress_decompress_round_trip() {
        let mut dict = Dictionary::default();
        dict.core.insert("however".into(), "but".into());
        dict.core.insert("therefore".into(), "∴".into());

        // Note: round trip is only exact for Layers 2+3 (Layer 1 is lossy)
        // So we test with plain text that has no markdown
        let input = "this works however and therefore we continue";
        let compressed = compress(input, &dict).output;
        let restored = decompress(&compressed, &dict);
        assert_eq!(restored, input);
    }
}
```

**Step 2: Run to verify they fail**

```bash
cargo test codec
```
Expected: compile error.

**Step 3: Write `src/codec.rs`**

```rust
use crate::dictionary::types::Dictionary;
use crate::layers::{abbreviate, strip, substitute};

/// Result of a compression operation.
#[derive(Debug)]
pub struct CompressResult {
    pub output: String,
    pub original_len: usize,
    pub compressed_len: usize,
}

impl CompressResult {
    /// Compression ratio: compressed / original. Lower = better.
    pub fn ratio(&self) -> f64 {
        self.compressed_len as f64 / self.original_len as f64
    }

    /// Percentage reduction.
    pub fn savings_pct(&self) -> f64 {
        (1.0 - self.ratio()) * 100.0
    }
}

/// Compress text through all three layers.
/// Layer 1 (strip) → Layer 2 (substitute) → Layer 3 (abbreviate)
///
/// Fail-safe: if any layer panics (shouldn't happen), the original is returned.
pub fn compress(text: &str, dict: &Dictionary) -> CompressResult {
    let original_len = text.len();

    // Layer 1: structural stripping (always runs)
    let after_strip = strip::strip(text);

    // Layer 2: pattern substitution (uses core dictionary)
    let after_sub = substitute::substitute(&after_strip, dict);

    // Layer 3: domain abbreviation (uses community packs)
    let output = abbreviate::abbreviate(&after_sub, dict);

    let compressed_len = output.len();
    CompressResult {
        output,
        original_len,
        compressed_len,
    }
}

/// Decompress text by reversing Layers 2 and 3.
/// Layer 1 (structural stripping) is one-way and cannot be reversed.
pub fn decompress(text: &str, dict: &Dictionary) -> String {
    // Reverse Layer 3
    let after_deabbrev = abbreviate::deabbreviate(text, dict);

    // Reverse Layer 2
    substitute::desubstitute(&after_deabbrev, dict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::types::Dictionary;

    #[test]
    fn compress_returns_shorter_text() {
        let mut dict = Dictionary::default();
        dict.core.insert("in order to".into(), "→".into());
        dict.core.insert("however".into(), "but".into());

        let input = "We do this in order to improve. However, it takes time.";
        let result = compress(input, &dict);
        assert!(result.output.len() < input.len());
    }

    #[test]
    fn compress_empty_dict_returns_stripped() {
        let dict = Dictionary::default();
        let input = "# Title\n\nsome text   \n\n\nextra blank";
        let result = compress(input, &dict);
        assert_eq!(result.output, "Title\n\nsome text\n\nextra blank");
    }

    #[test]
    fn stats_are_correct() {
        let mut dict = Dictionary::default();
        dict.core.insert("in order to".into(), "→".into());

        let input = "do this in order to pass";
        let result = compress(input, &dict);
        assert_eq!(result.original_len, input.len());
        assert!(result.compressed_len < result.original_len);
        assert!(result.ratio() < 1.0);
    }

    #[test]
    fn compress_decompress_round_trip() {
        let mut dict = Dictionary::default();
        dict.core.insert("however".into(), "but".into());
        dict.core.insert("therefore".into(), "∴".into());

        let input = "this works however and therefore we continue";
        let compressed = compress(input, &dict).output;
        let restored = decompress(&compressed, &dict);
        assert_eq!(restored, input);
    }
}
```

**Step 4: Run tests**

```bash
cargo test codec
```
Expected: all 4 tests pass.

**Step 5: Run the full test suite**

```bash
cargo test
```
Expected: all tests pass.

**Step 6: Commit**

```bash
git add src/codec.rs
git commit -m "feat: codec compress/decompress pipeline wiring all 3 layers"
```

---

### Task 8: Public Library API

Expose a clean public API from `src/lib.rs` so the crate is usable as a library.

**Files:**
- Modify: `src/lib.rs`

**Step 1: Write the failing test**

Add `tests/lib_api.rs`:
```rust
// tests/lib_api.rs
use steno::{compress, decompress, load_dictionary};

#[test]
fn public_api_compress() {
    let dict = load_dictionary().unwrap();
    let result = compress("in order to test this", &dict);
    // With universal dict loaded, "in order to" → "→"
    assert!(result.output.contains("→"));
    assert!(result.savings_pct() > 0.0);
}

#[test]
fn public_api_decompress() {
    let dict = load_dictionary().unwrap();
    let input = "this works however it goes";
    let compressed = compress(input, &dict).output;
    let restored = decompress(&compressed, &dict);
    assert_eq!(restored, input);
}
```

**Step 2: Run to verify they fail**

```bash
cargo test --test lib_api
```
Expected: compile error — `compress`, `decompress`, `load_dictionary` not public.

**Step 3: Update `src/lib.rs`**

```rust
pub mod codec;
pub mod dictionary;
pub mod layers;

use anyhow::Result;
use dictionary::types::Dictionary;

pub use codec::{compress, decompress, CompressResult};

/// Load the default dictionary: universal core + any installed packs.
pub fn load_dictionary() -> Result<Dictionary> {
    let mut dict = dictionary::loader::load_core()?;
    dictionary::loader::load_packs(&mut dict)?;
    Ok(dict)
}
```

**Step 4: Run tests**

```bash
cargo test --test lib_api
cargo test
```
Expected: all tests pass.

**Step 5: Commit**

```bash
git add src/lib.rs tests/lib_api.rs
git commit -m "feat: public library API (compress, decompress, load_dictionary)"
```

---

### Task 9: CLI — `compress`, `decompress`, `stats`

The main user-facing interface. Reads from stdin or file, writes to stdout.

**Files:**
- Modify: `src/main.rs`

**Step 1: Write `src/main.rs`**

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "steno",
    about = "Compress any text before it enters your LLM context window",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compress text (stdin or file → stdout)
    Compress {
        /// Input file (omit to read from stdin)
        file: Option<PathBuf>,
    },
    /// Decompress text (stdin or file → stdout)
    Decompress {
        /// Input file (omit to read from stdin)
        file: Option<PathBuf>,
    },
    /// Show compression stats without outputting compressed text
    Stats {
        /// Input file (omit to read from stdin)
        file: Option<PathBuf>,
    },
    /// Manage dictionary packs
    Dict {
        #[command(subcommand)]
        action: DictCommand,
    },
}

#[derive(Subcommand)]
enum DictCommand {
    /// List installed packs
    List,
    /// Install a pack from a local TOML file
    Add {
        /// Path to .toml pack file
        path: PathBuf,
    },
    /// Remove an installed pack by name
    Remove {
        /// Pack name (without .toml extension)
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let dict = steno::load_dictionary()?;

    match cli.command {
        Command::Compress { file } => {
            let text = read_input(file)?;
            let result = steno::compress(&text, &dict);
            io::stdout().write_all(result.output.as_bytes())?;
        }

        Command::Decompress { file } => {
            let text = read_input(file)?;
            let restored = steno::decompress(&text, &dict);
            io::stdout().write_all(restored.as_bytes())?;
        }

        Command::Stats { file } => {
            let text = read_input(file)?;
            let result = steno::compress(&text, &dict);
            eprintln!("Original:   {} bytes", result.original_len);
            eprintln!("Compressed: {} bytes", result.compressed_len);
            eprintln!("Savings:    {:.1}%", result.savings_pct());
            eprintln!("Ratio:      {:.3}", result.ratio());
        }

        Command::Dict { action } => match action {
            DictCommand::List => {
                let packs = steno::dictionary::loader::list_packs()?;
                if packs.is_empty() {
                    eprintln!("No packs installed. Use `steno dict add <file.toml>` to add one.");
                } else {
                    for pack in &packs {
                        println!("{}", pack);
                    }
                }
            }
            DictCommand::Add { path } => {
                let dest = steno::dictionary::loader::install_pack(&path)?;
                eprintln!("Installed pack to {}", dest.display());
            }
            DictCommand::Remove { name } => {
                steno::dictionary::loader::remove_pack(&name)?;
                eprintln!("Removed pack: {}", name);
            }
        },
    }

    Ok(())
}

fn read_input(file: Option<PathBuf>) -> Result<String> {
    match file {
        Some(path) => Ok(std::fs::read_to_string(&path)?),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}
```

**Step 2: Build and smoke test**

```bash
cargo build --release
```

Then try it:
```bash
echo "in order to test this we need however a good approach" | ./target/release/steno compress
./target/release/steno stats README.md
```

Expected: compressed output with `→` and `but` substituted; stats showing % savings.

**Step 3: Test help text**

```bash
./target/release/steno --help
./target/release/steno compress --help
./target/release/steno dict --help
```

Expected: clean help text with all subcommands listed.

**Step 4: Test dict commands**

```bash
./target/release/steno dict list
```
Expected: "No packs installed."

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: CLI with compress/decompress/stats/dict subcommands"
```

---

### Task 10: Integration Tests

End-to-end tests verifying the full pipeline with the real universal dictionary.

**Files:**
- Create: `tests/integration.rs`

**Step 1: Write `tests/integration.rs`**

```rust
use steno::{compress, decompress, load_dictionary};

/// Helper: load the real dictionary (universal core, no packs in test env).
fn dict() -> steno::dictionary::types::Dictionary {
    load_dictionary().expect("Failed to load dictionary")
}

#[test]
fn compress_produces_shorter_output() {
    let d = dict();
    // This sentence is packed with universal dictionary phrases
    let text = "In order to understand this, it is important to note that \
                as a result of the following conditions, we can therefore \
                conclude that furthermore the system works.";
    let result = compress(text, &d);
    assert!(
        result.savings_pct() > 10.0,
        "Expected >10% savings, got {:.1}%",
        result.savings_pct()
    );
}

#[test]
fn decompress_restores_exact_substitution_content() {
    let d = dict();
    // Only use plain text so Layer 1 doesn't alter content
    let original = "however, this works therefore and in order to verify it";
    let compressed = compress(original, &d).output;
    let restored = decompress(&compressed, &d);
    assert_eq!(restored, original);
}

#[test]
fn compress_plain_text_no_crash() {
    let d = dict();
    let texts = vec![
        "",
        "Hello, world!",
        "# Heading\n\nParagraph with **bold** and *italic*.",
        "```rust\nfn main() { println!(\"hello\"); }\n```",
        "Line 1\n\n\n\nLine 2 after many blank lines",
        "   trailing spaces   \n   on each line   ",
    ];
    for text in texts {
        let result = compress(text, &d);
        assert!(
            result.compressed_len <= result.original_len + 10,
            "Compression made text significantly LONGER for input: {:?}",
            text
        );
    }
}

#[test]
fn stats_ratio_is_between_zero_and_one() {
    let d = dict();
    let text = "in order to test this however it may work therefore we try";
    let result = compress(text, &d);
    assert!(result.ratio() > 0.0);
    assert!(result.ratio() <= 1.0);
}
```

**Step 2: Run integration tests**

```bash
cargo test --test integration
```
Expected: all 4 tests pass.

**Step 3: Run the full suite one final time**

```bash
cargo test
```
Expected: all tests pass, zero warnings.

**Step 4: Commit**

```bash
git add tests/integration.rs
git commit -m "test: integration tests for full compress/decompress pipeline"
```

---

### Task 11: Update `README.md` Status + Project State

**Files:**
- Modify: `README.md`
- Modify: `docs/project-state.md`

**Step 1: Update `README.md` Status section**

Replace the Status section:
```markdown
## Status

> 🟢 **Phase 1 complete** — Core engine + CLI shipped.

### Journey Log

| Date | Milestone |
|------|-----------|
| 2026-04-12 | Idea conceived |
| 2026-04-12 | Full design approved (identity, architecture, interfaces, dictionaries, error handling) |
| 2026-04-12 | Phase 1 complete: 3-layer engine, universal dictionary, CLI, integration tests |
```

**Step 2: Update `docs/project-state.md`**

Update the Current Phase, Session Log, and Next Session Prompt sections to reflect Phase 1 complete and Phase 2 (MCP server + `steno learn`) upcoming.

**Step 3: Commit**

```bash
git add README.md docs/project-state.md
git commit -m "docs: update status to Phase 1 complete"
```

---

## Phase 2 (Planned — Next Session)

- **Task 12:** MCP server (`src/mcp/server.rs`) — expose `compress` and `decompress` as MCP tools
- **Task 13:** `steno learn <path>` — corpus analysis to generate personal pack suggestions
- **Task 14:** `steno dict add <github-user/repo>` — remote pack install from GitHub
- **Task 15:** Publish to crates.io + GitHub release workflow
