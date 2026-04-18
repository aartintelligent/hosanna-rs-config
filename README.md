# hosanna-rs-config

[![Crates.io](https://img.shields.io/crates/v/hosanna-rs-config.svg)](https://crates.io/crates/hosanna-rs-config)
[![Docs.rs](https://img.shields.io/docsrs/hosanna-rs-config)](https://docs.rs/hosanna-rs-config)
[![CI](https://github.com/aartintelligent/hosanna-rs-config/actions/workflows/ci.yml/badge.svg)](https://github.com/aartintelligent/hosanna-rs-config/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/aartintelligent/hosanna-rs-config/graph/badge.svg)](https://codecov.io/gh/aartintelligent/hosanna-rs-config)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue.svg)](#minimum-supported-rust-version)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

A small, opinionated configuration layer for Rust services.

Declare a `Deserialize` struct for each component (HTTP server, database pool, message bus, feature-flag store — whatever you wire), implement [`ComponentConfig`](https://docs.rs/hosanna-rs-config/latest/hosanna_rs_config/trait.ComponentConfig.html) on it, and get uniform JSON + environment-variable loading, post-deserialisation validation, and a typed async builder interface for free. The same three-trait contract applies whether your binary has one component or thirty.

## Why

Ad-hoc `std::env::var().parse()` everywhere is where configuration bugs come to live. This crate is the single, small layer that enforces the same loading discipline across every component you configure:

- **Two sources, one rule:** a committed JSON file for defaults, environment variables for the deployment override. Env always wins.
- **Non-blocking file:** the JSON file is optional — a container image without `config/*.json` is a valid deployment.
- **Explicit validation:** `ComponentConfig::validate` runs after deserialisation and rejects values that the type system cannot express (URL scheme, port ranges, feature-flag combinations).
- **Builder symmetry:** every component implements `ComponentBuilder`, so wiring a binary looks the same regardless of which crate it imports from.
- **Strict hygiene:** no `unsafe` in production code, no free functions in the public surface, no `unwrap()`, no `as` in `use` statements.

## Install

```toml
[dependencies]
hosanna-rs-config = "0.1"
serde             = { version = "1", features = ["derive"] }
```

Consumer crates that implement `ComponentBuilder` will also need `async-trait` and `anyhow` in their own `Cargo.toml` — those types appear in the trait signature.

## Quick start

```rust,no_run
use hosanna_rs_config::{ComponentConfig, ComponentConfigLoader};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct NatsConfig {
    url: String,
    stream: String,
}

impl ComponentConfig for NatsConfig {
    fn env_prefix() -> &'static str { "NATS" }
    fn config_file() -> Option<&'static str> { Some("config/nats") }

    fn validate(&self) -> Result<(), String> {
        if !self.url.starts_with("nats://") {
            return Err(format!("invalid NATS url: {}", self.url));
        }
        Ok(())
    }
}

let cfg: NatsConfig = ComponentConfigLoader::<NatsConfig>::new().load()?;
# Ok::<_, hosanna_rs_config::ConfigError>(())
```

Environment variables use the double-underscore separator by default: `NATS__URL`, `NATS__STREAM`, `NATS__TLS__CERT_PATH`.

## Building a component

```rust,no_run
use async_trait::async_trait;
use hosanna_rs_config::{ComponentBuilder, ComponentConfig};
use serde::Deserialize;
use std::sync::Arc;

# #[derive(Debug, Deserialize)]
# struct NatsConfig { url: String }
# impl ComponentConfig for NatsConfig {
#     fn env_prefix() -> &'static str { "NATS" }
# }
pub trait NatsBus: Send + Sync {}

pub struct NatsClientBuilder;

#[async_trait]
impl ComponentBuilder for NatsClientBuilder {
    type Output = Arc<dyn NatsBus>;
    type Config = NatsConfig;

    async fn build(_config: NatsConfig) -> anyhow::Result<Self::Output> {
        // async_nats::connect(&_config.url).await?; …
        # unimplemented!()
    }
}
```

## Public surface

| Item                      | Kind   | Role                                                                 |
| ------------------------- | ------ | -------------------------------------------------------------------- |
| `ComponentConfig`         | trait  | Declared on each component's `Deserialize` struct. Says where its env/file sources live and how to validate the result. |
| `ComponentConfigLoader`   | struct | Stateless builder. Reads the sources declared by `ComponentConfig`, in priority order, and returns a validated value. |
| `ComponentBuilder`        | trait  | Async trait that turns a validated `Config` into the live component (`Output`). Returns `anyhow::Result`. |
| `ConfigError`             | enum   | `Load { component, source }` (parse/IO failure) + `Validation { component, reason }` (business rule). |

## Source priority

| Precedence | Source                  | Condition for being read                               |
| ---------- | ----------------------- | ------------------------------------------------------ |
| 1 (lowest) | JSON file               | `ComponentConfig::config_file` returns `Some`, and the file exists. Missing file is **not** an error. |
| 2          | Environment variables   | Always read. Prefix and separator come from `env_prefix()` / `env_separator()`. |

Values from a later source override values from an earlier one on a per-field basis — you can keep sensible defaults in JSON and override exactly the two fields you care about from the environment.

## Minimum supported Rust version

This crate requires **Rust 1.85** (the stabilisation of edition 2024). Bumping the MSRV is treated as a minor-version change.

## Development

```bash
cargo test -- --test-threads=1   # env-var tests mutate process state; keep them serial
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --all -- --check
cargo doc --no-deps --open       # check the rustdoc examples
```

A git `pre-commit` hook is wired via [`cargo-husky`](https://github.com/rhysd/cargo-husky). The hook source lives in `.cargo-husky/hooks/pre-commit` and is copied into `.git/hooks/` automatically the first time you run `cargo test` in a fresh clone. It runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, and `cargo doc` before every commit. In an emergency you can bypass it with `git commit --no-verify`.

Changelog entries are authored with [Changie](https://github.com/miniscruff/changie): add an entry under `.changes/unreleased/` instead of editing `CHANGELOG.md` by hand.

## License

Licensed under the Apache License, Version 2.0 ([`LICENSE`](LICENSE) or <https://www.apache.org/licenses/LICENSE-2.0>).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.
