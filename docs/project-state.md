# Steno — Project State

This file is updated at the end of every session. When resuming, read this first.

---

## Current Phase

**Phase 4 — Community Infrastructure** (COMPLETE)

All 39 tests passing. Phase 4 delivered:
- `.github/workflows/release.yml` — GitHub Actions release pipeline (4 targets: linux/macos-arm/macos-intel/windows)
- `docs/contributing-dictionaries.md` — full dictionary contribution guide
- `dictionaries/community/steno-dict-code.toml` — reference community pack (programming terms)
- `CONTRIBUTING.md` — code + dictionary contribution guide
- `.github/ISSUE_TEMPLATE/` — bug, feature, and dictionary pack issue templates
- `src/gain.rs` + `steno gain` command — cumulative compression stats (inspired by RTK gain)
- `steno gain` records every compress run and shows total bytes/% saved

---

## Design Progress

| Section | Status |
|---------|--------|
| Identity & Positioning | ✅ Approved |
| Architecture | ✅ Approved |
| Interfaces (CLI / library / MCP + optional RTK/MemPalace) | ✅ Approved |
| Dictionary design (3-tier + open publishing) | ✅ Approved |
| Error handling & testing strategy | ✅ Approved |

**Design doc:** `docs/plans/2026-04-12-steno-design.md`

---

## Key Decisions Locked

- **Language:** Rust — single binary, zero dependencies
- **Compression layers:** 3 in sequence: structural strip → pattern substitution → domain abbreviation
- **Reversible:** Yes — full decompress back to original always possible
- **Dictionary format:** TOML — human-readable, diff-friendly, PR-friendly
- **Interfaces:** CLI + Rust crate + MCP server (user chooses what to use)
- **Distribution model:** Open source; community contributes domain dictionary packs
- **Standalone:** No integrations with other tools — steno is its own product
- **Dictionary growth model:** Ships with universal core; community packs expand it (steno-dict-code, steno-dict-medical, etc.); users can run `steno learn` for personal extensions

---

## Folder Structure

```
D:\Claude\steno\
  README.md               ← living project document, updated every session
  docs/
    project-state.md      ← this file — resume point for new sessions
    design/               ← design docs (written after design is fully approved)
    plans/                ← implementation plans (written after design)
```

---

## Next Session Prompt

To resume, tell Claude:
> "Let's start working on the steno project"

Claude will read this file and `README.md`, then continue from where we left off.

---

## Session Log

| Date | What happened |
|------|---------------|
| 2026-04-12 | Project started. Full design completed and approved across 5 sections. Implementation plan next. |
| 2026-04-12 | Phase 1 complete — Cargo.toml, 3 layers (strip/substitute/abbreviate), dictionary types + core bundle, codec pipeline with header hashing. 28 tests passing. |
| 2026-04-12 | Phase 2 complete — CLI (compress/decompress/stats/dict), platform-aware config paths, dictionary loader, codec builder. 36 tests passing. GitHub Actions CI for Windows/Linux/macOS. |
| 2026-04-12 | Phase 3 complete — MCP server (steno serve) with steno_compress/steno_decompress/steno_stats tools via rmcp 1.x. stdio transport, tracing to stderr. |
| 2026-04-12 | Phase 4 complete — release pipeline, dict contribution guide, steno-dict-code community pack, steno gain command. 39 tests passing. |
