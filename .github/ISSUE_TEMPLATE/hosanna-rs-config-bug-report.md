---
name: Bug report
about: Report a defect in hosanna-rs-config — incorrect behaviour, broken invariant, or compilation failure.
title: "[bug] <short summary>"
labels: ["bug", "triage"]
assignees: []
---

<!--
⚠ If your report concerns a security-sensitive issue — a leak of a
secret value through a logged `ConfigError`, a parsing path that reads
uncontrolled input, or any way to observe a validated configuration
without going through `ComponentConfigLoader::load` — do NOT file a
public issue. Open a private GitHub security advisory instead. See
SECURITY.md and CONTRIBUTING.md.
-->

## Summary

<!-- One or two sentences: what goes wrong, and why you think it's a bug. -->

## Environment

- **Crate version:** `hosanna-rs-config = "x.y.z"`
- **Rust version:** `rustc --version` output here
- **Target triple:** `rustc -vV | grep host` output here (e.g. `x86_64-unknown-linux-gnu`)
- **OS:** Linux / macOS / Windows / …

## Minimal reproducer

<!--
Please reduce the problem to the smallest possible `main.rs` or `#[test]`.
A typical reproducer sets a few env vars, calls the loader, and asserts.
-->

```rust
use hosanna_rs_config::{ComponentConfig, ComponentConfigLoader};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Repro { /* … */ }

impl ComponentConfig for Repro {
    fn env_prefix() -> &'static str { "REPRO" }
}

fn main() -> Result<(), hosanna_rs_config::ConfigError> {
    // …
    let _cfg: Repro = ComponentConfigLoader::<Repro>::new().load()?;
    Ok(())
}
```

## Expected behaviour

<!-- What you expected to happen, and why (pointer to rustdoc, README contract table, etc.). -->

## Actual behaviour

<!-- What actually happens: error message, wrong value, panic, miscompile. Paste verbatim. -->

```text
<stdout / stderr / compiler output here>
```

## Does this touch a load-bearing invariant?

<!--
Tick every box that applies. These are the guarantees the loader
commits to — a "yes" on any of them makes the issue higher priority.
-->

- [ ] Missing JSON file is no longer silently ignored
- [ ] Environment variables no longer override JSON
- [ ] `ComponentConfig::validate` is no longer invoked before return
- [ ] `ConfigError` surfaces the wrong `component` label
- [ ] `ComponentConfigLoader` leaked a secret value into an error message
- [ ] None of the above — this is a functional / ergonomic bug

## Additional context

<!-- Logs, screenshots, links to related issues, prior art, or anything else useful. -->
