# Security Policy

`hosanna-rs-config` sits at the edge of every binary that uses it: it reads the environment and parses JSON, and its output is then fed into network clients, database pools, and other privileged resources. Bugs here are structural, not cosmetic — please treat reports accordingly.

## Supported versions

Only the latest minor line receives security fixes. Pre-1.0 releases follow semantic versioning with the caveat that breaking changes can ship in any minor bump.

| Version | Supported          |
| ------- | ------------------ |
| `0.1.x` | ✅ security fixes  |
| `< 0.1` | ❌ no support      |

## What counts as a vulnerability

Please report privately if you can demonstrate any of the following:

- A code path that leaks a raw configuration value (password, token, API key) into a `ConfigError`, a panic message, or a `Debug` output. The loader should never echo back input values in diagnostics.
- A malformed JSON or environment value that causes the loader to `panic!`, abort, or stack-overflow instead of returning a `ConfigError`.
- A way to have `ComponentConfig::validate` silently skipped — for example, by constructing a configuration through an alternative code path that bypasses [`ComponentConfigLoader::load`].
- A TOCTOU or symlink issue in the JSON-file path that lets a co-located process read a configuration intended for another component.
- Any addition of `unsafe` outside of the single sanctioned test-only block around `std::env::set_var`, or any compile-time escape from the crate's `deny(unsafe_code)` lint.
- A `From`, `Deref`, or similar trait implementation on a configuration wrapper that widens the surface unintentionally.

Bugs that do **not** qualify as vulnerabilities (they are still welcome as public issues):

- A `ConfigError` message could be more informative without touching a secret.
- Compilation failure on a new toolchain.
- Broken intra-doc links, typos in rustdoc.
- Ergonomic gaps (missing builder method, missing `From` impl on a non-secret type).

## How to report

**Do not open a public GitHub issue for anything in the "vulnerability" list above.**

Preferred channel — a private GitHub Security Advisory:

1. Go to the repository's **Security** tab.
2. Click **Report a vulnerability**.
3. Fill in the form with a reproducer.

Alternative channel — email the maintainer listed in [`Cargo.toml`](Cargo.toml) under `authors`. Use the subject line `[security][hosanna-rs-config]` so the message is easy to route.

Include, at minimum:

- The crate version and commit SHA you tested against.
- The Rust toolchain version (`rustc --version`).
- A minimal reproducer — ideally a `#[test]` that fails on main. Redact or fabricate any real secret before attaching.
- Your assessment of the severity and of any partial mitigations.

If you want, tell us how you'd like to be credited in the advisory (name / handle / link, or anonymous).

## What to expect from us

- **Acknowledgement within 72 hours.** Weekends and public holidays may push this slightly; you will at least receive a "received, looking at it" message.
- **Triage within 7 days.** We confirm (or refute) the issue, agree on severity, and propose an embargo window.
- **Fix + coordinated disclosure.** We prefer to ship the patch release and the public advisory together. Default embargo is 30 days from the confirmed triage; we are happy to adjust based on severity and on the reporter's constraints.
- **CVE assignment.** For anything that genuinely breaks one of the documented invariants we request a CVE through GitHub's advisory tooling.

We will keep you informed at each step and credit you in the advisory and in the `CHANGELOG.md` entry unless you explicitly ask to remain anonymous.

## Scope

This policy covers the `hosanna-rs-config` crate itself. Vulnerabilities in upstream dependencies (`config`, `serde`, `thiserror`, `anyhow`, `async-trait`) should be reported to their respective projects; if the issue is exploitable *through* this crate in a way that would not exist in the dependency alone, please still let us know.

[`ComponentConfigLoader::load`]: https://docs.rs/hosanna-rs-config/latest/hosanna_rs_config/struct.ComponentConfigLoader.html#method.load
