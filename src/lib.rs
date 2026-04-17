#![cfg_attr(not(test), forbid(unsafe_code))]
//! Shared configuration infrastructure for the `hosanna-rs-*` family of
//! crates.
//!
//! This crate provides the two traits and the loader that every
//! downstream component plugs into in order to obtain a uniform
//! configuration pipeline: JSON file for committed defaults,
//! environment variables for per-deployment overrides, business-rule
//! validation, and a typed builder that turns the validated value into
//! a live component (a client, a pool, a bus, …).
//!
//! # The three moving parts
//!
//! - [`ComponentConfig`] — implemented on each component's
//!   `Deserialize` struct. It declares the environment-variable prefix,
//!   the optional JSON file path, and the `validate` rules.
//! - [`ComponentConfigLoader`] — stateless builder that reads the
//!   sources in priority order (env vars override JSON) and returns a
//!   validated `C: ComponentConfig`.
//! - [`ComponentBuilder`] — the async trait that turns the validated
//!   config into the live component.
//!
//! # Source priority
//!
//! From lowest to highest precedence:
//!
//! 1. JSON file at the path returned by
//!    [`ComponentConfig::config_file`] — silently skipped if the file
//!    does not exist.
//! 2. Environment variables prefixed with
//!    [`ComponentConfig::env_prefix`] and separated by
//!    [`ComponentConfig::env_separator`] (defaults to `"__"`).
//!
//! # Example — declaring and loading a component config
//!
//! ```rust,no_run
//! use hosanna_rs_config::{ComponentConfig, ComponentConfigLoader};
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct ServerConfig {
//!     host: String,
//!     port: u16,
//! }
//!
//! impl ComponentConfig for ServerConfig {
//!     fn env_prefix() -> &'static str { "SERVER" }
//!     fn config_file() -> Option<&'static str> { Some("config/server") }
//!
//!     fn validate(&self) -> Result<(), String> {
//!         if self.port < 1024 {
//!             return Err(format!("port {} is reserved", self.port));
//!         }
//!         Ok(())
//!     }
//! }
//!
//! // Reads `config/server.json` first, then overrides with
//! // `SERVER__HOST` / `SERVER__PORT` from the environment.
//! let cfg: ServerConfig = ComponentConfigLoader::<ServerConfig>::new().load()?;
//! # Ok::<_, hosanna_rs_config::ConfigError>(())
//! ```
//!
//! # Project conventions
//!
//! The crate is authored under a strict style that callers can rely on:
//! no `unsafe` in production code, no free functions in the public
//! surface (every function is a method on a struct or a trait), no
//! `unwrap()`, no `as` in `use` statements, and explicit typing on
//! every public signature.
//!
//! [`ComponentConfig::env_prefix`]: crate::ComponentConfig::env_prefix
//! [`ComponentConfig::env_separator`]: crate::ComponentConfig::env_separator
//! [`ComponentConfig::config_file`]: crate::ComponentConfig::config_file

pub mod error;
pub mod traits;
pub mod types;

pub use error::ConfigError;
pub use traits::ComponentBuilder;
pub use traits::ComponentConfig;
pub use types::ComponentConfigLoader;
