# Contributing Dictionaries

Anyone can create and publish a steno dictionary pack. No permission needed — the community discovers quality packs organically.

---

## Quick Start

### 1. Create a TOML file

```toml
[meta]
name        = "steno-dict-mypack"
description = "Short description of what domain this covers"
author      = "your-github-username"
version     = "0.1.0"
language    = "en"

[entries]
"your verbose phrase"         = "short"
"another long pattern"        = "abbr"
"it is recommended that you"  = "rec:"
```

### 2. Test it locally

```bash
steno dict add ./steno-dict-mypack.toml
echo "your verbose phrase" | steno compress
echo "your verbose phrase" | steno compress | steno decompress
```

The second command should reproduce the original text exactly. If it doesn't, there's a conflict in your entries.

### 3. Publish to GitHub

Create a repo named `steno-dict-<name>` (e.g. `steno-dict-code`, `steno-dict-medical`).

Put your TOML file at the root as `dict.toml`.

### 4. Users install it

```bash
steno dict add ./dict.toml
```

---

## Dictionary Quality Guidelines

These are guidelines, not rules. The community decides what's good.

### DO

- **Make abbreviations readable by an LLM without a decoder.** `"machine learning"` → `"ML"` is good. `"machine learning"` → `"x7q"` is not.
- **Prefer symbols and short words over cryptic codes.** `"therefore"` → `"∴"` is readable in context.
- **Target truly verbose phrases.** The best entries are phrases you'd never write concisely yourself but that appear constantly in your domain.
- **Test round-trip fidelity.** Every entry must compress and decompress back to the original exactly.
- **Include the `[meta]` block** so users know what they're installing.

### AVOID

- **Single-word abbreviations.** `"the"` → `"t"` breaks LLM comprehension.
- **Abbreviations that collide with common symbols.** If `"→"` is used in code, don't use it for text substitution.
- **Overly broad patterns.** `"is"` → `"i"` would corrupt code, identifiers, and prose.
- **Domain-specific terms in the universal core.** If it only applies to one field, it belongs in a community pack.

---

## Entry Format Rules

Each entry must satisfy:

1. **Pattern** (left side): the full phrase to match. Case-insensitive during matching.
2. **Code** (right side): the short replacement. Must be unique across all entries in the file.
3. **No substring conflicts:** `"in order to"` and `"in order"` in the same dict can cause ambiguous matches. Use one or the other.
4. **No empty strings.** Both pattern and code must be non-empty.

```toml
[entries]
# Good — clear, readable, unique codes
"in order to"                  = "→"
"it is important to note that" = "‼"
"with respect to"              = "re:"
"as a result of"               = "∴"

# Bad — cryptic, too short, breaks prose
"the"    = "t"       # Too broad
"is"     = "i"       # Too broad
"result" = "r7x"     # Cryptic code
```

---

## Naming Convention

| Pack name | Domain |
|-----------|--------|
| `steno-dict-code` | Programming, stack traces, error messages |
| `steno-dict-medical` | Clinical notes, diagnoses, pharmacology |
| `steno-dict-legal` | Contracts, case law, legal documents |
| `steno-dict-science` | Research papers, methodology, citations |
| `steno-dict-finance` | Financial reports, market language |
| `steno-dict-<yourdomain>` | Whatever you need |

---

## Contributing to the Core Dictionary

The universal core (`dictionaries/core/universal.toml`) accepts PRs for entries that are:

- **Truly universal** — appear across ALL domains and LLM workflows, not just one field
- **Safe** — won't collide with code, identifiers, or structured data
- **High-frequency** — phrases that appear constantly in natural language prompts

Domain-specific terms belong in community packs, not core. When in doubt, make a community pack.

**To submit a core entry:**
1. Fork the repo
2. Add your entry to `dictionaries/core/universal.toml`
3. Run `cargo test` — the round-trip test will catch any conflicts
4. Open a PR with a brief explanation of why the entry is universal

---

## Testing Your Dictionary

Before publishing, run these checks:

```bash
# 1. Install your pack
steno dict add ./your-pack.toml

# 2. Test compression
echo "your test phrase" | steno compress

# 3. Test round-trip fidelity (output must match input exactly)
echo "your test phrase" | steno compress | steno decompress

# 4. Test stats
echo "your test phrase" | steno stats

# 5. Test with a real document from your domain
cat sample-document.txt | steno compress | steno decompress > restored.txt
diff sample-document.txt restored.txt  # should be empty
```

---

## Example Packs

See `dictionaries/community/` for reference implementations:

- `steno-dict-code.toml` — programming and software engineering terms

---

## Questions?

Open an issue on GitHub with the label `dictionary`.
