# Steno Phase 5 Design — v1.1.0

**Date:** 2026-05-12  
**Status:** Approved — ready for implementation planning  
**Scope:** Core dictionary expansion (20 → ~200 entries) + suggest_code() collision fix

---

## Problem

Actual prose savings (25–40%) fall well short of design target (40–70%). Root cause: `universal.toml` has only 20 entries vs the 200–500 target. Secondary issue: `suggest_code()` generates initials with no dedup — `steno suggest --add` silently overwrites conflicting codes.

**Use-case priority (user-confirmed):** RTK companion (C) > Conversation history (D) > Manual compress (B) > WebFetch hook (A)

---

## Approach

Option 1 selected: **Dict blitz + collision fix → ship v1.1.0**

Dict expansion closes ~80% of the savings gap immediately. Collision fix prevents data loss on `suggest --add`. Strip layer, context detection, and GitHub install deferred to a future phase.

---

## Component 1 — Core Dictionary Expansion

**File:** `dictionaries/core/universal.toml`  
**Change:** 20 → ~200 entries across 8 categories

| Category | Target | Focus |
|---|---|---|
| LLM reasoning/hedging | 40 | "it's worth noting that", "as mentioned above", "as a result of" |
| Verbose filler (AI-speak) | 30 | "I'd be happy to help", "great question", "certainly", "of course" |
| Transition language | 25 | "on the other hand", "furthermore", "nevertheless", "in contrast" |
| Causal connectives | 20 | "given that", "in light of", "owing to the fact that" |
| Temporal phrases | 20 | "at this point in time", "going forward", "in the near future" |
| Discourse markers | 20 | "to put it simply", "in a nutshell", "to summarize", "that being said" |
| Technical prose | 25 | "with respect to", "in the context of", "in the implementation" |
| Conversation meta | 20 | "as I mentioned earlier", "as we discussed", "to clarify" |

**Ordering:** longest-to-shortest within each category block — matches the longest-match-first substitution in `substitute.rs`.

**Validation:** run `deemuk stats` on a ~500-word Claude conversation sample before and after. Target: savings climb from 25–40% toward 45–55% on prose.

---

## Component 2 — suggest_code() Collision Fix

**File:** `src/learn.rs`

### New signature

```rust
pub fn suggest_code(phrase: &str, existing_codes: &HashSet<String>) -> String
```

### Logic

1. Generate initials (current behavior)
2. If code exists in `existing_codes`, try `code + "2"`, `code + "3"` … up to `code + "9"`
3. If all 9 taken, return the `"9"` variant with a warning flag

### suggest command output (new)

```
"machine learning model"    → mlm     (new)
"maximum likelihood method" → mlm2    (renamed, mlm taken)
"multi-layer model"         → mlm3    (renamed, mlm mlm2 taken)
```

With `--add`: writes renamed code, prints `[renamed: mlm → mlm2]`. No silent data loss.

### Scope boundary

Collision detection runs in `suggest` flow only. `dict personal-add "phrase" "code"` stays manual — user is explicitly choosing the code.

---

## Component 3 — Testing

New tests in `src/learn.rs`:

| Test | Assertion |
|---|---|
| `test_suggest_code_no_collision` | No existing codes → returns initials |
| `test_suggest_code_collision_once` | One collision → appends `"2"` |
| `test_suggest_code_collision_chain` | mlm through mlm4 taken → returns mlm5 |
| `test_suggest_output_shows_rename` | Display includes `(renamed: mlm → mlm2)` |

---

## Release

- `Cargo.toml`: `1.0.0` → `1.1.0`
- `CHANGELOG.md`: dict expansion + collision-safe suggest
- `cargo publish` after all tests pass

---

## Out of Scope (future phase)

- Strip layer code indentation normalization
- Context detection (`--mode prose|code|mixed`)
- GitHub dict pack installation (`deemuk dict add github:...`)
