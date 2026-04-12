# Steno — Design Document

**Date:** 2026-04-12
**Status:** Approved — ready for implementation planning

---

## Overview

`steno` is an open source Rust-based text compression tool for LLM context. It compresses any text before it enters the LLM context window using three sequential layers inspired by stenographic shorthand systems. The community-maintained dictionary is the core product — it grows and improves as more users contribute domain packs.

---

## Section 1: Identity & Positioning

**Name:** `steno`
**Tagline:** *Compress anything going into your LLM. Less tokens, same meaning.*

steno is standalone — it works with zero external dependencies. Users may optionally pair it with RTK (command output compression) and/or MemPalace (conversation memory), but neither is required.

| Configuration | What's compressed | Typical savings |
|---|---|---|
| steno only | Documents, prompts, conversation history | 40-70% |
| steno + RTK | Above + command output | 70-90% |
| steno + MemPalace | Above + memory retrieval | 60-85% |
| steno + RTK + MemPalace | Everything | 75-95% |

---

## Section 2: Architecture

```
steno/
  src/
    main.rs              ← CLI entry point
    lib.rs               ← public library API (the crate)
    layers/
      strip.rs           ← Layer 1: structural stripping
      substitute.rs      ← Layer 2: pattern substitution
      abbreviate.rs      ← Layer 3: domain abbreviation
    dictionary/
      core.rs            ← bundled universal dictionary (compiled in)
      loader.rs          ← loads community packs + user extensions
      learner.rs         ← corpus analysis for `steno learn`
    mcp/
      server.rs          ← MCP server exposing compress/decompress tools
    codec.rs             ← compress + decompress pipeline orchestration
  dictionaries/
    core/
      universal.toml     ← universal pattern dictionary (ships with binary)
    community/           ← domain packs (separate repos, pulled in)
  docs/
  tests/
```

**Compression pipeline:**
```
Input text
    ↓
[Layer 1] Structural stripping    — remove markdown noise, whitespace, boilerplate
    ↓
[Layer 2] Pattern substitution    — common LLM phrases → short codes (core dictionary)
    ↓
[Layer 3] Domain abbreviation     — domain-specific terms → community/personal abbreviations
    ↓
[Header]  Dictionary snapshot     — hash of exact dict state for guaranteed decompression
    ↓
Compressed output
```

**Dictionary format — TOML:**
```toml
[meta]
name        = "steno-dict-example"
description = "Example domain dictionary"
author      = "username"
version     = "0.1.0"
language    = "en"

[entries]
"in order to"                   = "→"
"it is important to note that"  = "‼"
"as mentioned above"            = "↑ref"
```

Human-readable, diff-friendly, easy to PR.

**Compression header:**
Every compressed output carries a header with a dictionary hash for guaranteed round-trip fidelity:
```
[steno:v1:dict=a3f9c2b1]
compressed text here...
```

---

## Section 3: Interfaces

### CLI
```bash
steno compress <file>           # compress a file
steno compress < input.txt      # pipe-friendly stdin
steno decompress <file>         # decompress back to original
steno learn <path>              # build personal dictionary from corpus
steno dict add <user/repo>      # install community domain pack
steno dict list                 # list installed packs
steno dict update               # update all packs
steno stats <file>              # show compression ratio
steno bench <file>              # benchmark compression + savings
steno serve                     # start MCP server
```

### Rust Library (crate)
```rust
use steno::{Codec, DictionarySet};

let dict = DictionarySet::default()
    .with_user_dict("~/.steno/user.toml");

let codec = Codec::new(dict);
let compressed = codec.compress(text)?;
let original   = codec.decompress(compressed)?;

println!("Saved {}%", compressed.ratio());
```

### MCP Server
```bash
steno serve    # starts on stdio
```
Exposes: `steno_compress`, `steno_decompress`, `steno_stats`
Compatible with: Claude Code, Cursor, Windsurf, any MCP-compatible client.

---

## Section 4: Dictionary Architecture

### Three Tiers

**Tier 1 — Universal core** (compiled into the binary)
~200-500 entries. Patterns universal across all LLM workflows. Maintained by steno core team. Strict PR review — only truly universal patterns accepted.

**Tier 2 — Community domain packs** (published by anyone, installed via CLI)
```
steno-dict-code       → programming, stack traces, errors
steno-dict-science    → papers, methodology, citations
steno-dict-legal      → contracts, clauses, case law
steno-dict-medical    → clinical notes, diagnoses, drug names
steno-dict-finance    → reports, filings, market terminology
```
Published as `github-username/steno-dict-<name>`. No approval required. Anyone creates, anyone publishes.

**Tier 3 — Personal extension** (`~/.steno/user.toml`)
Built by `steno learn <path>`. Scans user's corpus, proposes abbreviations, user approves. Private — never shared.

### Dictionary Creation (Open to All)
1. Create `dict.toml` with `[meta]` + `[entries]` sections
2. Publish to GitHub as `username/steno-dict-<name>`
3. Users install with `steno dict add username/steno-dict-<name>`

Community packs run automated round-trip CI via GitHub Actions template steno provides.

---

## Section 5: Error Handling & Testing

**Philosophy:** Never silently corrupt text.

```
compress(text)   → Ok(CompressedOutput) | Ok(Original)      // fail-safe
decompress(text) → Ok(Original) | Err(DictionaryMismatch)   // fail-loud
```

| Scenario | Behaviour |
|---|---|
| Dictionary pack missing at decompress | Hard error with install instructions |
| Layer fails mid-pipeline | Skip layer, continue, warn user |
| Input already compressed | Detect header, return as-is with warning |

**Testing — three levels:**
1. **Unit tests** — every core dictionary entry has a round-trip test
2. **Integration tests** — real-world samples (markdown, code, transcripts), byte-for-byte fidelity
3. **Community CI** — every dict pack PR runs automated round-trip tests via provided GitHub Actions template

**Benchmarks:** `steno bench` tracks compression ratio and token savings, reported in CI.

---

## Implementation Phases

| Phase | Scope |
|---|---|
| Phase 1 | Core Rust crate: codec pipeline, 3 layers, universal dictionary, compress/decompress |
| Phase 2 | CLI: all commands, dictionary management, `steno learn` |
| Phase 3 | MCP server |
| Phase 4 | Community infrastructure: CI template, dictionary contribution guide, docs site |
