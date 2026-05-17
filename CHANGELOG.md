# Changelog

## [1.1.0] - 2026-05-16

### Added

- **Core dictionary expansion** — `dictionaries/core/universal.toml` grew from 20 to ~210 entries across 8 labeled categories: LLM reasoning/hedging, verbose filler (AI-speak), transition language, causal connectives, temporal phrases, discourse markers, technical prose, and conversation meta. Expected prose savings improve from 25–40% toward 45–55%.
- **Collision-safe `suggest_code()`** — `deemuk suggest` and `deemuk suggest --add` now detect code conflicts. When a generated code is already taken (e.g., both "machine learning model" and "maximum likelihood method" would produce `mlm`), the second gets `mlm2`, the third `mlm3`, and so on up to `mlm9`. Renamed codes are annotated in output: `[renamed: mlm → mlm2]`. No more silent overwrite of personal.toml entries.
- **Test coverage** — 4 new tests for `suggest_code()` collision handling (50 total, up from 46).

### Fixed

- `test_missing_file_returns_default` flaky test in `gain.rs` — isolated each test to its own temp path via atomic counter, eliminating a race condition when tests ran in parallel.

## [1.0.0] - 2026-04-16

Initial release. Three-layer compression (strip → substitute → abbreviate), CLI (`compress`, `decompress`, `stats`, `gain`, `learn`, `suggest`, `dict`, `serve`), MCP server, and community dictionary packs for code, science, medical, and AI-social domains.
