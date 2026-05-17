# Steno Improvements — Brainstorm Continuation

**Date:** 2026-05-12  
**Status:** Brainstorming in progress — clarifying questions phase  
**Resume by:** Reading this file, then proceeding to "Next Steps" section below

---

## Current State (as of 2026-05-12)

- **Published:** `deemuk v1.0.0` live on crates.io — `cargo install deemuk`
- **GitHub:** https://github.com/deemuk123/steno (master, all 4 phases complete)
- **Binary name:** `deemuk` / **Lib name:** `steno` (crate renamed; steno was taken)
- **Tests:** 46 passing across 4 phases
- **Phases complete:** Phase 1 (core crate) → Phase 2 (CLI) → Phase 3 (MCP server) → Phase 4 (release pipeline + community infra)

---

## Identified Improvement Gaps

These were discovered by reading the full codebase. All 5 are confirmed against actual source files.

### Gap 1 — Thin Core Dictionary (ROOT CAUSE of savings gap)

**File:** `dictionaries/core/universal.toml`  
**Current:** 20 entries  
**Design target:** 200–500 entries (from `docs/plans/2026-04-12-steno-design.md`)  
**Impact:** This is the single biggest lever. The 25–40% actual savings vs 40–70% designed savings is almost entirely explained by this gap.

Sample current entries:
```toml
"in order to"                        = "→"
"it is important to note that"       = "‼"
"for example"                        = "e.g."
"in other words"                     = "i.e."
"in conclusion"                      = "∎"
"due to the fact that"               = "because"
```

Every high-frequency LLM phrase added here benefits every single compression run.

### Gap 2 — Strip Layer Too Minimal

**File:** `src/layers/strip.rs`  
**Current:** Only collapses multiple blank lines + trims trailing whitespace  
**Missing:** Code indentation normalization — repeated indentation in code blocks consumes tokens without meaning. Consistent normalization to 2-space or tab-to-2-space conversion would reduce code token count.  
**Risk:** Must not change semantics for any code input — this is tricky.

### Gap 3 — suggest_code() Has No Collision Deduplication

**File:** `src/learn.rs`  
**Current code:**
```rust
pub fn suggest_code(phrase: &str) -> String {
    phrase.split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect()
}
```
**Problem:** Generates only initials. "machine learning model" → "mlm", "maximum likelihood method" → also "mlm". When `steno suggest --add` auto-writes both to personal.toml, the second silently overwrites the first. User loses substitutions without warning.  
**Fix:** Add collision detection — check existing dict before suggesting; append numeric suffix (`mlm2`) or skip with warning.

### Gap 4 — No Context Detection

**Current:** Same compression pipeline applied regardless of input type  
**Missing:** Detection for code vs prose vs structured data. Different strategies would save more in each context. Example: code blocks don't benefit from phrase substitution; prose doesn't benefit from indentation normalization.  
**Complexity:** Medium — requires heuristics at codec level; opt-in flag is safer than auto-detect.

### Gap 5 — No GitHub Dict Pack Installation

**Current:** Community packs require manual file copy to `%APPDATA%\steno\dicts\`  
**Design target:** `steno dict add github:deemuk123/steno-dict-science` one-liner  
**Missing:** HTTP fetch + validate + install flow. `steno dict list --remote` to browse available packs.  
**Complexity:** Low-medium. curl + TOML validation + copy to config dir.

---

## Brainstorming Skill Progress

Following the `superpowers:brainstorming` skill process:

| Step | Status |
|------|--------|
| 1. Explore project context | ✅ Complete |
| 2. Ask clarifying questions | 🔄 In progress — see below |
| 3. Propose 2–3 approaches | ⬜ Pending |
| 4. Present design sections | ⬜ Pending |
| 5. Write design doc | ⬜ Pending |
| 6. Invoke writing-plans skill | ⬜ Pending |

---

## Clarifying Questions Asked / Answered

### Q1: Primary use case (answered 2026-05-12)

**Question asked:** Where are you running steno the most right now?  
- A) Hook → WebFetch outputs  
- B) Manual compress on prompt text  
- C) RTK companion  
- D) Conversation history  

**User's priority order: C > D > B > A**  
- **C (RTK companion) is highest priority** — steno handles prose/narrative that RTK doesn't (RTK = command output; steno = everything else)
- D (conversation history) — second priority: compressing long conversation context before it enters context window
- B (manual compress on prompt text) — third
- A (WebFetch hook) — lowest priority

**Implication for design:** Focus improvements on making steno a better passive companion to RTK for prose/narrative/conversation contexts. Core dictionary expansion is the highest-value investment. GitHub dict install comes second (makes it easy to deploy richer packs). suggest_code collision fix is a quality-of-life correctness fix.

### Q2: (NOT YET ASKED)

Next question to ask: How much of the core dict gap to address at once?  
Options to present:
- A) Big batch: add 150–200 high-frequency LLM phrases in one session
- B) Focused: add 50 most impactful phrases targeting conversation/prose patterns
- C) Automated: use `steno learn` on a corpus of LLM conversation logs first, then bulk-add suggestions

---

## Recommended Approach (to be confirmed after Q2)

Based on C > D > B > A priority:

**Phase 5a — Core Dictionary Expansion (immediate, high ROI)**
- Target: 20 → 200 entries in `universal.toml`
- Focus on: LLM conversation patterns, reasoning phrases, common prose connectives
- Use `steno learn` on existing conversation corpus to identify real high-frequency phrases
- Estimated savings improvement: 25–40% → 40–60%

**Phase 5b — GitHub Dict Install (enables ecosystem)**
- `steno dict add github:<user>/<repo>` command
- Unlocks easy deployment of richer community packs

**Phase 5c — suggest_code() Collision Fix (correctness)**
- Add collision check in `src/learn.rs`
- Numeric suffix dedup: `mlm`, `mlm2`, `mlm3`

**Phase 5d — Context Detection (optional, future)**
- Opt-in `--mode prose|code|mixed` flag
- Auto-detect as stretch goal

---

## Next Steps to Resume

1. Ask Q2 (see above)
2. Based on Q2 answer, propose 2–3 concrete approaches with trade-offs
3. Present design (scaled to complexity of chosen approach)
4. Write design doc to `docs/plans/2026-05-12-steno-phase5-design.md`
5. Invoke `superpowers:writing-plans` skill

---

## Windows Dev Environment (for quick reference)

```bash
# In Git Bash session — must prefix cargo:
PATH="/c/ProgramData/mingw64/mingw64/bin:$PATH" "/c/Users/Dell/.cargo/bin/cargo" build

# In new terminal — just:
cargo build

# Publish to crates.io:
cargo publish  # already live as deemuk v1.0.0
```

Personal dict: `%APPDATA%\steno\personal.toml`  
Community packs: `%APPDATA%\steno\dicts\`
