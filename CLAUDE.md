# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Purpose

`hosanna-rs-config` is a **standalone repository** (single crate, no `[workspace]`). It publishes the shared configuration infrastructure consumed by the `hosanna-rs-*` family of crates. Consumers implement two traits from this crate and get uniform JSON + environment-variable loading, post-deserialisation validation, and a typed async builder entry-point.

The source + rustdoc are the canonical description of the contract — start from the crate-level doc in `src/lib.rs`, then dive into `traits.rs` / `types.rs` / `error.rs` as needed. `README.md` covers the public-facing surface.

## Project-wide coding conventions (enforced)

- **No `as` in `use` statements** — rename at the definition site.
- **No free functions** — every function is a method on a struct or trait.
- **Explicit typing on all public items.**
- **No `unwrap()` in production code**; `expect("…")` is fine only in tests.
- **No `unsafe` in production** — `#![cfg_attr(not(test), forbid(unsafe_code))]` at the crate root. Test modules downgrade to `#[allow(unsafe_code)]` **only** to wrap edition-2024 `std::env::set_var`. That is the single sanctioned `unsafe` in the entire crate; a second one is a review red flag.
- **Config format is JSON only**, and a missing file must never error.
- `edition = "2024"`, MSRV `1.85`.

## Architecture

Flat module layout under `src/`, matching the `hosanna-rs-secret` convention (`error` / `traits` / `types`):

- `traits.rs` — both public traits live here.
  - `ComponentConfig`: consumers declare `env_prefix()` (required) and optionally override `env_separator()` (default `"__"`), `config_file()` (default `None`), and `validate()` (default `Ok`).
  - `ComponentBuilder`: async trait with `Output` / `Config` associated types. `build` returns `anyhow::Result<Output>`.
- `types.rs` — `ComponentConfigLoader<C>`. `new()` seeds from the trait; `with_*` methods override per-call. `load()` composes sources, runs `validate()`, wraps failures in `ConfigError`. Internally sets both `prefix_separator` and `separator` on `config::Environment` to `C::env_separator()` — the single knob drives both the prefix→key split and nested-key splits. Inline `#[cfg(test)]` tests live at the bottom of this file.
- `error.rs` — `ConfigError::Load { component, source }` and `ConfigError::Validation { component, reason }`. Both carry `component` = `C::env_prefix()` for log routing.
- `lib.rs` — crate-level rustdoc (the canonical description of the contract), module declarations, flat re-exports.

### Source priority

1. JSON file — read only if `config_file()` is `Some` and the file exists. Missing file = silent skip.
2. Environment variables — always read, highest precedence.

Later sources override earlier ones **per field**, so a JSON file can hold defaults and the environment can override individual keys.

## Common commands

```bash
cargo test -- --test-threads=1               # env-var tests mutate process state
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --all -- --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked

# Run a single test:
cargo test loads_from_env_variables -- --test-threads=1
```

**Always use `--test-threads=1` for the full test suite.** The loader tests mutate process-wide environment variables, so parallel scheduling causes cross-test contamination. The prefix discipline (`LOADER_SIMPLE__*`, `LOADER_OVERRIDE__*`, `LOADER_STRICT__*`, `LOADER_MISSING__*`) reduces collisions but is not sufficient on its own — preserve the prefix pattern if you add new tests, and keep them serial.

## Tooling and release

- **Pre-commit hook** at `.cargo-husky/hooks/pre-commit`, installed by `cargo-husky` on first `cargo test`. Runs fmt / clippy / test / doc — same gates as CI.
- **Changelog** via [Changie](https://github.com/miniscruff/changie): add fragments under `.changes/unreleased/` rather than editing `CHANGELOG.md` directly. Kinds and bumps are configured in `.changie.yaml`.
- **CI** (`.github/workflows/ci.yml`): fmt, clippy, rustdoc (`-D warnings`), test matrix (stable + MSRV 1.85) with `--test-threads=1`, and a non-blocking coverage job.
- **Release** (`.github/workflows/release.yml`): triggered by `vX.Y.Z` tags. Verifies tag-vs-`Cargo.toml` version agreement, verifies `.changes/<ver>.md` exists, re-runs tests, publishes to crates.io, creates a GitHub Release. All preparation (`changie batch`, version bump) must happen on `main` **before** tagging.
- **Changelog gate** (`.github/workflows/changelog-check.yml`): every PR must add a fragment under `.changes/unreleased/*.yaml`, carry the `skip-changelog` label, or only touch docs/CI/licence.

## Commit messages

Conventional Commits v1.0.0. The `feat` / `fix` / `perf` / `revert` types map to a Changie kind and require a fragment; `refactor` / `docs` / `test` / `chore` / `ci` / `build` / `style` do not. Breaking changes use `!` or a `BREAKING CHANGE:` footer.
