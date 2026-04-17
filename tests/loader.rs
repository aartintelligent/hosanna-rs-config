//! Integration tests for [`ComponentConfigLoader`].
//!
//! All tests share a single [`TestConfig`] type so that every method on
//! the generic [`ComponentConfigLoader`] is exercised through exactly
//! one monomorphization. That keeps the `instantiations` metric of
//! `cargo llvm-cov` at 100 % without artificial duplication.
//!
//! The tests mutate process-wide environment variables, so they must be
//! run with `--test-threads=1` (enforced in CI and in the pre-commit
//! hook). Each test uses a unique `LOADER_*` prefix so that, even under
//! that discipline, a misbehaving test cannot silently feed values into
//! another case.

#![allow(unsafe_code)] // edition 2024 requires `unsafe` around `std::env::set_var`.

use hosanna_rs_config::{ComponentConfig, ComponentConfigLoader, ConfigError};
use serde::Deserialize;

// ── Shared fixture ───────────────────────────────────────────────────────────

/// Single generic parameter used by every loader test. Validation
/// rejects `port == 1` as a sentinel, so changing the port value is the
/// only thing needed to flip between the `Ok` and `Err` branches of
/// [`ComponentConfig::validate`].
#[derive(Debug, Deserialize)]
struct TestConfig {
    url: String,
    port: u16,
}

impl ComponentConfig for TestConfig {
    fn env_prefix() -> &'static str {
        "LOADER_DEFAULT"
    }

    fn validate(&self) -> Result<(), String> {
        if self.port == 1 {
            Err(format!("port {} is reserved", self.port))
        } else {
            Ok(())
        }
    }
}

/// Minimal companion fixture that inherits *all* trait defaults. One
/// dedicated test exercises it so that the `env_separator`,
/// `config_file`, and — crucially — the default `validate` methods on
/// `ComponentConfig` stay covered despite [`TestConfig`] overriding
/// `validate`.
#[derive(Debug, Deserialize)]
struct DefaultsCfg {
    url: String,
    port: u16,
}

impl ComponentConfig for DefaultsCfg {
    fn env_prefix() -> &'static str {
        "LOADER_TRAITDEF"
    }
}

// ── Source composition ──────────────────────────────────────────────────────

#[test]
fn loads_from_env_variables() {
    unsafe {
        std::env::set_var("LOADER_DEFAULT__URL", "http://localhost");
        std::env::set_var("LOADER_DEFAULT__PORT", "8080");
    }

    let config = ComponentConfigLoader::<TestConfig>::new()
        .load()
        .expect("should load from env");

    assert_eq!(config.url, "http://localhost");
    assert_eq!(config.port, 8080);
}

#[test]
fn trait_defaults_are_applied_when_not_overridden() {
    unsafe {
        std::env::set_var("LOADER_TRAITDEF__URL", "http://traitdef");
        std::env::set_var("LOADER_TRAITDEF__PORT", "7000");
    }

    let config = ComponentConfigLoader::<DefaultsCfg>::new()
        .load()
        .expect("default validate/env_separator/config_file must work");

    assert_eq!(config.url, "http://traitdef");
    assert_eq!(config.port, 7000);
}

#[test]
fn missing_json_file_is_not_blocking() {
    unsafe {
        std::env::set_var("LOADER_MISSING__URL", "http://localhost");
        std::env::set_var("LOADER_MISSING__PORT", "9090");
    }

    let config = ComponentConfigLoader::<TestConfig>::new()
        .with_env_prefix("LOADER_MISSING")
        .with_config_file("this/path/does/not/exist")
        .load()
        .expect("missing json file must not fail the load");

    assert_eq!(config.url, "http://localhost");
    assert_eq!(config.port, 9090);
}

#[test]
fn env_prefix_can_be_overridden() {
    unsafe {
        std::env::set_var("LOADER_OVERRIDE__URL", "http://override");
        std::env::set_var("LOADER_OVERRIDE__PORT", "1234");
    }

    let config = ComponentConfigLoader::<TestConfig>::new()
        .with_env_prefix("LOADER_OVERRIDE")
        .load()
        .expect("should load with overridden prefix");

    assert_eq!(config.url, "http://override");
    assert_eq!(config.port, 1234);
}

#[test]
fn env_separator_can_be_overridden() {
    // With a single-underscore separator, the prefix/key split happens
    // after one `_`, so `LOADER_SEP_URL` is decomposed as
    // prefix `LOADER_SEP` + key `URL`.
    unsafe {
        std::env::set_var("LOADER_SEP_URL", "http://sep");
        std::env::set_var("LOADER_SEP_PORT", "5000");
    }

    let config = ComponentConfigLoader::<TestConfig>::new()
        .with_env_prefix("LOADER_SEP")
        .with_env_separator("_")
        .load()
        .expect("custom separator must work");

    assert_eq!(config.url, "http://sep");
    assert_eq!(config.port, 5000);
}

#[test]
fn default_impl_matches_new() {
    unsafe {
        std::env::set_var("LOADER_DEF__URL", "http://default");
        std::env::set_var("LOADER_DEF__PORT", "3000");
    }

    let config = <ComponentConfigLoader<TestConfig> as Default>::default()
        .with_env_prefix("LOADER_DEF")
        .load()
        .expect("Default impl must produce a usable loader");

    assert_eq!(config.url, "http://default");
    assert_eq!(config.port, 3000);
}

// ── Error variants ──────────────────────────────────────────────────────────

#[test]
fn validation_error_is_propagated() {
    unsafe {
        std::env::set_var("LOADER_STRICT__URL", "http://strict");
        std::env::set_var("LOADER_STRICT__PORT", "1");
    }

    let err = ComponentConfigLoader::<TestConfig>::new()
        .with_env_prefix("LOADER_STRICT")
        .load()
        .expect_err("validation must reject the sentinel port");

    // `matches!` avoids a defensive match arm that would otherwise
    // remain uncovered in this test. The `component` label is the
    // *trait's* prefix (`TestConfig::env_prefix()`), not the override.
    assert!(matches!(
        &err,
        ConfigError::Validation { component, reason }
            if *component == "LOADER_DEFAULT" && reason.contains('1')
    ));
    // Exercise Display, not only Debug, so the thiserror-generated
    // formatter is covered.
    assert!(err.to_string().contains("validation failed"));
}

#[test]
fn load_error_on_unparseable_env_value() {
    unsafe {
        std::env::set_var("LOADER_BADENV__URL", "http://bad");
        std::env::set_var("LOADER_BADENV__PORT", "not-a-number");
    }

    let err = ComponentConfigLoader::<TestConfig>::new()
        .with_env_prefix("LOADER_BADENV")
        .load()
        .expect_err("an unparseable port must yield a Load error");

    assert!(matches!(
        &err,
        ConfigError::Load { component, .. } if *component == "LOADER_DEFAULT"
    ));
    assert!(err.to_string().contains("failed to load"));
}
