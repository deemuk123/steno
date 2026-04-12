# Contributing to Steno

Thank you for your interest in contributing. There are two distinct ways to contribute:

1. **Dictionary packs** — no Rust required, just TOML
2. **Core code** — Rust, standard open source workflow

---

## Dictionary Contributions (No Rust Needed)

The easiest way to contribute. See the full guide at [docs/contributing-dictionaries.md](docs/contributing-dictionaries.md).

### Quick version

Create a TOML file with your domain phrases:

```toml
[meta]
name        = "steno-dict-mypack"
description = "What domain this covers"
author      = "your-github-username"
version     = "0.1.0"
language    = "en"

[entries]
"your verbose phrase" = "short"
```

Test round-trip fidelity, publish as a GitHub repo named `steno-dict-<name>`, put the TOML at the root as `dict.toml`.

**To add to the core dictionary** (`dictionaries/core/universal.toml`): entries must be truly universal across all domains, safe around code/identifiers, and high-frequency. Open a PR with a brief explanation.

---

## Code Contributions

### Setup

```bash
git clone https://github.com/deemuk123/steno.git
cd steno
cargo build
cargo test
```

All 36 tests must pass before submitting.

### Workflow

1. Fork the repo and create a branch (`git checkout -b feat/my-feature`)
2. Make your changes
3. Run `cargo test` — all tests must pass
4. Run `cargo clippy` — no warnings
5. Open a PR against `master`

### What we accept

- Bug fixes with a regression test
- New CLI subcommands (discuss in an issue first)
- Performance improvements with benchmarks
- New dictionary entries to `dictionaries/core/universal.toml`
- Documentation improvements

### What we don't accept (without prior discussion)

- Breaking changes to the public crate API
- New dependencies without strong justification
- Features that belong in a community dictionary pack

---

## Code Style

- Standard `rustfmt` formatting (`cargo fmt` before committing)
- No `unwrap()` in library code — use `?` or return `StenoError`
- Tests live in the same file as the code they test (inline `#[cfg(test)]` modules)
- Commit messages: imperative mood, present tense (`add`, `fix`, `update`)

---

## Questions?

Open an issue with the appropriate label:
- `bug` — something broken
- `feature` — new capability request
- `dictionary` — dictionary pack question
