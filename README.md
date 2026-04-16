# steno

> Compress anything going into your LLM. Less tokens, same meaning.

---

## What is steno?

`steno` is a Rust-based text compression tool for LLM context. It sits between your content and your LLM, making every token count.

Unlike RTK (which compresses command output) and MemPalace (which manages conversation memory), steno compresses **any** text before it reaches the LLM context window — documents, wiki pages, conversation history, prompts, source files.

---

## The Problem

Every token you send to an LLM costs money and burns context window. Most of that text is padded with:
- Structural boilerplate (markdown decoration, redundant whitespace)
- Common verbose phrases ("in order to", "it is important to note that")
- Repeated domain terms that could be abbreviated

RTK solves this for command output. Nothing solves it for everything else.

---

## How It Works

steno applies three compression layers in sequence:

```
Input text
    ↓
[Layer 1] Structural stripping    — remove markdown noise, whitespace, boilerplate
    ↓
[Layer 2] Pattern substitution    — common LLM phrases → short codes (universal dictionary)
    ↓
[Layer 3] Domain abbreviation     — domain-specific terms → personalized/community abbreviations
    ↓
Compressed output → LLM context window
```

All three layers are fully reversible — steno can decompress back to the original.

---

## The Dictionary

The dictionary is the product. steno ships with a **universal core dictionary** covering common LLM patterns. The community contributes **domain packs**:

| Pack | Coverage | Install |
|------|----------|---------|
| `steno-dict-code` | Programming, stack traces, dev workflow | built-in |
| `steno-dict-science` | Research papers, methodology, LLM benchmarks | built-in |
| `steno-dict-medical` | Clinical notes, diagnoses, pharmacology | built-in |
| `steno-dict-legal` | Legal documents, contracts, case law | community |
| `steno-dict-finance` | Markets, economics, financial reports | community |
| *(your domain)* | Anything | `steno dict add ./your-pack.toml` |

Users can also run `steno learn <path>` to build a **personal extension dictionary** from their own corpus. The more users contribute, the better compression gets for everyone.

---

## Where It Fits

steno is a **standalone tool**. It works on its own with zero dependencies. Optionally pair it with RTK and/or MemPalace for compounding token savings — but neither is required.

### steno alone
```
Your content  ──→  steno compress  ──→  LLM context window
```
Compresses documents, wiki pages, prompts, conversation history before they reach the LLM. Best for: anyone feeding large text into an LLM who wants fewer tokens.

---

### steno + RTK *(optional)*
```
Bash commands  ──→  RTK  ──→  compact command output  ──→  LLM
Your text      ──→  steno ──→  compressed prose        ──→  LLM
```
RTK handles structured command output (test results, git diffs, build logs). steno handles unstructured prose. Together they cover every token source in a developer workflow. **Combined savings: 70-95%** across the full context window.

Best for: developers using Claude Code or similar AI coding assistants.

---

### steno + MemPalace *(optional)*
```
Past sessions  ──→  MemPalace  ──→  relevant memories (semantic search)
                                         ↓
Your content   ──→  steno compress  ──→  LLM context window
```
MemPalace retrieves only the memories relevant to your current session. steno compresses those memories further before they enter context. Without steno, MemPalace memories arrive verbose. With steno, the same memories use 60-80% fewer tokens.

Best for: anyone using MemPalace for long-running AI sessions or personal knowledge bases.

---

### steno + RTK + MemPalace *(full stack)*
```
Bash output   ──→  RTK        ──→  ┐
Past memories ──→  MemPalace  ──→  ├──→  steno compress  ──→  LLM
Documents     ──→  steno      ──→  ┘
```
Every token source is compressed. Command output, memory retrieval, and document context all arrive at the LLM in their most compact form. This is the maximum token efficiency configuration.

Best for: power users running long, complex AI sessions where context window and cost are real constraints.

---

| Configuration | What's compressed | Typical savings |
|---|---|---|
| steno only | Documents, prompts, conversation history | 40-70% |
| steno + RTK | Above + command output | 70-90% |
| steno + MemPalace | Above + memory retrieval | 60-85% |
| steno + RTK + MemPalace | Everything | 75-95% |

> All integrations are optional. steno works perfectly on its own.

---

## Interfaces

steno ships as three things in one:

| Interface | Use case |
|-----------|----------|
| **CLI** | Pipe text through steno in any workflow |
| **Rust library (crate)** | Import steno into your own tools |
| **MCP server** | Native tool for Claude, Cursor, Windsurf, any MCP-compatible LLM |

---

## MCP Setup

steno works as an MCP server with Claude Code, Cursor, Windsurf, and any MCP-compatible client.

### Claude Code

Add to `~/.claude/settings.json` (or `.claude/settings.json` in your project):

```json
{
  "mcpServers": {
    "steno": {
      "command": "steno",
      "args": ["serve"]
    }
  }
}
```

### Cursor / Windsurf

Add to your MCP config file:

```json
{
  "mcpServers": {
    "steno": {
      "command": "steno",
      "args": ["serve"]
    }
  }
}
```

### Available MCP tools

| Tool | Description |
|------|-------------|
| `steno_compress` | Compress text to reduce token usage (20–70% savings) |
| `steno_decompress` | Restore compressed text to the original |
| `steno_stats` | Show savings % without modifying text |

All three tools accept `{ "text": "..." }` as input.

---

## Usage

### Install

```bash
# Rust users (recommended)
cargo install steno

# Prebuilt binaries — Linux, macOS, Windows
# Download from: https://github.com/deemuk123/steno/releases/latest
```

### Compress text

```bash
echo "in order to succeed you must for example practice" | steno compress
cat document.md | steno compress > document.steno
```

### Decompress

```bash
cat document.steno | steno decompress
steno decompress document.steno
```

### Check savings without compressing

```bash
cat document.md | steno stats
# Original:   1240 bytes
# Compressed:  891 bytes
# Saved:       28.1%
```

### Manage dictionary packs

```bash
steno dict list
steno dict add ./steno-dict-code.toml
steno dict remove steno-dict-code
```

### See cumulative compression savings

```bash
steno gain
# Steno gain report
# -----------------
# Runs:      47
# Original:  284,102 bytes
# Saved:     91,438 bytes  (32.2%)
```

Every `steno compress` run is automatically recorded. `steno gain` shows the totals.

### Learn from your own corpus

```bash
# Analyze any text file and track phrase frequencies
steno learn ./my-notes.md
steno learn ./docs/

# See top phrases not yet in any dictionary
steno suggest --min 5 --top 20

# Auto-add all suggestions to your personal dictionary
steno suggest --min 5 --add

# Or add one at a time
steno dict personal-add "your verbose phrase" "short-code"
```

The more you `learn`, the better `suggest` gets. Personal dictionary entries stack on top of all community packs.

---

## Quick Demo

```
Input (198 chars):
  Instead of explaining the concept to the audience in order to clarify
  the solution, start with the pain point rather than just the answer.
  This builds credibility by showing what actually matters.

steno compress → 143 chars (-28%):
  inst-of explaining the concept to the-aud →-to clarify
  the-soln, st-with the pain-pt r-than the answer.
  this-bld credibility by-show wh-actly matters.
```

Savings compound with domain packs — code-heavy content routinely hits 60–95%.

---

## Status

> 🟢 **v1.0.0** — Production ready. Full CLI, MCP server, community dictionaries, `steno learn` corpus builder, `steno gain` savings tracker.

### Previous milestones
> Phase 3 complete — MCP server with steno_compress, steno_decompress, steno_stats; works with Claude Code, Cursor, Windsurf.

### Journey Log

| Date | Milestone |
|------|-----------|
| 2026-04-12 | Idea conceived — stenography applied to LLM token compression |
| 2026-04-12 | Core concept approved: 3-layer compression + community dictionary + Rust + 3 interfaces |
| 2026-04-12 | Architecture approved: layered pipeline, TOML dictionary format, codec orchestration |
| 2026-04-12 | Interfaces approved: CLI + Rust crate + MCP server, standalone + optional RTK/MemPalace pairing |
| 2026-04-12 | Dictionary design approved: 3-tier system (core + community packs + personal), open publishing model |
| 2026-04-12 | Error handling approved: fail-safe compression, loud decompression errors, 3-level test strategy |
| 2026-04-12 | Full design complete — moving to implementation planning |
| 2026-04-12 | Phase 1 complete — core Rust crate with 3-layer pipeline, universal dictionary, full test coverage |
| 2026-04-12 | Phase 2 complete — full CLI with compress/decompress/stats/dict commands, cross-platform CI (Windows/Linux/macOS) |
| 2026-04-12 | Phase 3 complete — MCP server with steno_compress, steno_decompress, steno_stats tools; works with Claude Code, Cursor, Windsurf |
| 2026-04-12 | Phase 4 complete — release pipeline (GitHub Actions), contributing guide, steno-dict-code community pack, `steno gain` command |

---

## Architecture

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
      universal.toml     ← universal pattern dictionary (shipped with binary)
    community/           ← domain packs (separate repos, pulled in)
```

**Dictionary format:** TOML files — human-readable, diff-friendly, easy to PR.
```toml
# Example entry in universal.toml
"in order to" = "→"
"it is important to note that" = "‼"
"as mentioned above" = "↑"
"the following" = "ff:"
```

Each entry is fully reversible — steno stores the dictionary snapshot used to compress, so decompression is always exact.

---

## Contributing

### Creating Your Own Dictionary

Anyone can create and publish a steno dictionary pack. No permission needed.

**1. Create a TOML file:**
```toml
# steno-dict-mycomain/dict.toml
[meta]
name        = "steno-dict-mydomain"
description = "Abbreviations for [your domain]"
author      = "your-github-username"
version     = "0.1.0"
language    = "en"

[entries]
"your verbose phrase"    = "short"
"another long pattern"   = "abbr"
```

**2. Publish to GitHub** as `github-username/steno-dict-<name>`

**3. Users install it:**
```bash
steno dict add github-username/steno-dict-<name>
```

That's it. No approval needed. The community discovers quality packs organically.

### Dictionary Quality Guidelines *(not rules)*
- Each abbreviation should be readable by an LLM without a decoder
- Prefer symbols and short words over cryptic codes
- Include the `[meta]` block so users know what they're installing
- Test round-trip fidelity before publishing

### Contributing to the Core Dictionary
The universal core dictionary (`dictionaries/core/universal.toml`) accepts PRs. Entries must be truly universal — patterns that appear across all domains and all LLM workflows. Domain-specific terms belong in community packs, not core.

See [`docs/contributing-dictionaries.md`](docs/contributing-dictionaries.md) for full guidelines and [`CONTRIBUTING.md`](CONTRIBUTING.md) for code contributions.

---

## Error Handling

steno never silently corrupts text. If any compression layer fails, the original text is returned unchanged.

```
compress(text)   → Ok(CompressedOutput) | Ok(Original)      // never corrupts
decompress(text) → Ok(Original) | Err(DictionaryMismatch)   // fails loud if dict missing
```

| Scenario | Behaviour |
|---|---|
| Dictionary pack missing at decompress | Hard error — tells user which pack to install |
| Layer fails mid-pipeline | Skips that layer, continues, warns user |
| Input already compressed | Detects header, returns as-is with warning |

---

## Inspiration

- [Gregg Shorthand](https://en.wikipedia.org/wiki/Gregg_shorthand) — the original stenographic system
- [Pitman Shorthand](https://en.wikipedia.org/wiki/Pitman_shorthand) — phonetic compression for speed writing
- [Vannevar Bush's Memex](https://en.wikipedia.org/wiki/Memex) — associative trails through knowledge
