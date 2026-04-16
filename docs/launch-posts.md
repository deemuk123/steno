# deemuk v1.0.0 — Launch Posts

Use these when posting. Customize the benchmark numbers if you have fresher data.

---

## Hacker News — "Show HN"

**Title:**
```
Show HN: steno – compress any text before it hits your LLM context window (Rust, MIT)
```

**Body:**
```
deemuk sits between your content and your LLM and compresses text before it enters
the context window. Three-layer pipeline: structural stripping → pattern substitution
→ domain abbreviation. Fully reversible — the LLM sees compressed text and can
decompress it exactly.

Typical results: 25–40% on prose, 60–95% on domain-heavy content (code, medical,
research papers). Savings compound with domain packs.

Features:
- CLI, Rust crate, and MCP server (works with Claude Code, Cursor, Windsurf)
- Community dictionary packs: code, science, medical (more welcome)
- `deemuk learn <corpus>` builds a personal extension from your own text
- `deemuk gain` tracks cumulative savings across all runs

The dictionary is the product. Contributions welcome — any domain, any language.

cargo install deemuk
https://github.com/deemuk123/steno
```

---

## r/rust

**Title:**
```
[project] deemuk v1.0.0 – LLM token compressor built in Rust
```

**Body:**
```
Built a CLI tool that compresses text before it enters an LLM context window.

Three-layer pipeline:
1. Structural stripping — markdown noise, redundant whitespace
2. Pattern substitution — verbose phrases → short codes (universal dict)
3. Domain abbreviation — domain-specific terms via TOML community packs

Fully reversible. Ships as CLI + Rust crate + MCP server.

The interesting Rust bits:
- Longest-match substitution with word-boundary checks (Unicode-safe)
- Dict hash in compressed header for exact decompression verification
- `deemuk learn` uses n-gram frequency analysis to auto-suggest dict entries
- Cross-platform CI on Windows/Linux/macOS

cargo install deemuk
https://github.com/deemuk123/steno

Looking for contributors on domain packs (legal, finance, any language).
Dictionary format is TOML — 10 minute contribution.
```

---

## r/LocalLLaMA

**Title:**
```
steno: compress any text before sending to your LLM — 25-95% token savings, MIT, Rust
```

**Body:**
```
If you feed large documents, RAG chunks, wiki pages, or long system prompts to your
LLM, deemuk compresses them before they hit the context window.

Results from real usage:
- General prose: 25–40% savings
- Code + stack traces: 60–95% savings
- Domain-heavy content (medical, research): 40–80% savings

How it works:
- Three compression layers (structural strip → phrase substitution → domain abbrev)
- Fully reversible — include the steno header and the LLM can decompress exactly
- Community dictionary packs for code, science, medical (TOML format, easy to add)
- `deemuk learn <your-docs>` builds a personal dictionary from your own corpus
- MCP server for native Claude Code / Cursor / Windsurf integration

Works with any LLM. No API changes needed — compress before sending, decompress
the output if needed.

cargo install deemuk | https://github.com/deemuk123/steno
```

---

## X / Twitter thread

```
Tweet 1:
Shipped deemuk v1.0.0 — compress any text before it hits your LLM context window.

25-95% token savings. Fully reversible. MIT.

cargo install deemuk

🧵

Tweet 2:
Three compression layers:
1. Strip markdown noise + redundant whitespace
2. Replace verbose phrases with short codes ("in order to" → "→")
3. Apply domain abbreviations from community dictionary packs

All reversible. The LLM can decompress exactly.

Tweet 3:
The real power is community dictionaries.

Ships with packs for: code, science, medical.

Add your own in 10 minutes — it's just TOML:
"statistical significance" = "stat-sig"
"differential diagnosis"   = "DDx"

No permission needed. Publish to GitHub, share the link.

Tweet 4:
deemuk learn <your-corpus>

Analyzes your text files, tracks phrase frequencies, suggests what to add to
your personal dictionary.

Feed it your notes, docs, system prompts. Dictionary improves with your usage.

Tweet 5:
Also ships as an MCP server.

Works natively with Claude Code, Cursor, Windsurf.

Add to settings.json:
{ "deemuk": { "command": "deemuk", "args": ["serve"] } }

LLM can now call deemuk_compress / deemuk_decompress directly.

Tweet 6:
Built in Rust. MIT license. All platforms.

GitHub: https://github.com/deemuk123/steno
cargo install deemuk

PRs welcome — especially domain dictionary packs.
```
