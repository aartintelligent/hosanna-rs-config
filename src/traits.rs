//! The two traits every consumer crate implements.
//!
//! - [`ComponentConfig`] lives on the component's `Deserialize` struct
//!   and declares *where* its configuration comes from and *how* it is
//!   validated.
//! - [`ComponentBuilder`] lives on a dedicated zero-sized struct
//!   (`NatsClientBuilder`, `DatabasePoolBuilder`, …) and describes how
//!   to turn a validated configuration into the live component.
//!
//! Keeping both traits in the same module mirrors the shape of a
//! consumer crate: the `Config` and the `Builder` always travel
//! together.

use async_trait::async_trait;
use serde::Deserialize;

// ── ComponentConfig ──────────────────────────────────────────────────────────

/// Describes, for a single component, **where** its configuration comes
/// from and **how** it is validated.
///
/// The trait is deliberately declarative: implementers return plain
/// `&'static str` literals, never construct a loader themselves. The
/// construction side is handled by [`ComponentConfigLoader`], which
/// reads these associated functions to drive the [`config`] crate.
///
/// # Example
///
/// ```rust
/// use hosanna_rs_config::ComponentConfig;
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize)]
/// struct NatsConfig {
///     url: String,
///     stream: String,
/// }
///
/// impl ComponentConfig for NatsConfig {
///     fn env_prefix() -> &'static str { "NATS" }
///     fn config_file() -> Option<&'static str> { Some("config/nats") }
///
///     fn validate(&self) -> Result<(), String> {
///         if !self.url.starts_with("nats://") {
///             return Err(format!("invalid NATS url: {}", self.url));
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// # Defaults
///
/// - [`env_separator`](Self::env_separator) defaults to `"__"`, matching
///   the `Environment::separator` convention of the [`config`] crate.
/// - [`config_file`](Self::config_file) defaults to `None`: a component
///   with environment-only configuration does not need to override it.
/// - [`validate`](Self::validate) defaults to accepting any value;
///   override it to encode business rules that cannot be expressed in
///   the type system.
///
/// [`ComponentConfigLoader`]: crate::ComponentConfigLoader
pub trait ComponentConfig: for<'de> Deserialize<'de> + Send + Sync + 'static {
    /// Stable prefix used to filter environment variables for this
    /// component.
    ///
    /// Given a prefix of `"NATS"` and the default separator, the loader
    /// picks up `NATS__URL`, `NATS__STREAM`, …
    ///
    /// The value is also used as the `component` label on every
    /// [`ConfigError`](crate::error::ConfigError) emitted by the loader,
    /// so keep it short, uppercase, and unique across the fleet.
    fn env_prefix() -> &'static str;

    /// Separator used between the env prefix and each nested field name.
    ///
    /// Defaults to `"__"` because single underscores collide with
    /// snake-cased field names (e.g. `NATS_URL` would be ambiguous if
    /// the struct had a `nats_url` field at the top level). Override
    /// only if you know why you need to.
    fn env_separator() -> &'static str {
        "__"
    }

    /// Path to the JSON configuration file, **without** the `.json`
    /// extension.
    ///
    /// The loader marks the file as optional: if the path does not
    /// exist, no error is produced and the pipeline continues with the
    /// environment layer. Returning `None` (the default) skips the file
    /// layer entirely.
    fn config_file() -> Option<&'static str> {
        None
    }

    /// Business-rule validation applied **after** successful
    /// deserialisation and **before** the loader returns.
    ///
    /// Return `Err(reason)` to reject the configuration. The reason
    /// travels unchanged into
    /// [`ConfigError::Validation`](crate::error::ConfigError::Validation),
    /// so write it as something you would be happy to see in a log line.
    ///
    /// The default implementation accepts any value.
    fn validate(&self) -> Result<(), String> {
        let _ = self;
        Ok(())
    }
}

// ── ComponentBuilder ─────────────────────────────────────────────────────────

/// Builds a component (typically a network client or resource pool)
/// from its validated [`ComponentConfig`].
///
/// Each consumer crate implements this trait on a dedicated zero-sized
/// struct (`NatsClientBuilder`, `DatabasePoolBuilder`, …). That struct
/// is the stable, namespaced entry point — call it from `main` once the
/// loader has produced a value.
///
/// # Example
///
/// ```rust,no_run
/// use async_trait::async_trait;
/// use hosanna_rs_config::{ComponentBuilder, ComponentConfig};
/// use serde::Deserialize;
/// use std::sync::Arc;
///
/// #[derive(Debug, Deserialize)]
/// pub struct NatsConfig {
///     pub url: String,
/// }
///
/// impl ComponentConfig for NatsConfig {
///     fn env_prefix() -> &'static str { "NATS" }
/// }
///
/// pub trait NatsBus: Send + Sync {}
///
/// pub struct NatsClientBuilder;
///
/// #[async_trait]
/// impl ComponentBuilder for NatsClientBuilder {
///     type Output = Arc<dyn NatsBus>;
///     type Config = NatsConfig;
///
///     async fn build(config: NatsConfig) -> anyhow::Result<Self::Output> {
///         // … `async_nats::connect(&config.url).await?` and friends
///         # unimplemented!()
///     }
/// }
/// ```
///
/// # Why `anyhow::Result`
///
/// Component construction typically surfaces errors from very different
/// layers — the network, TLS, DNS, third-party crates — and the
/// top-level binary almost always erases them into a single chain
/// anyway. `anyhow::Error` captures that reality without forcing every
/// crate to invent its own enum. [`ComponentConfig::validate`] remains
/// responsible for the *structured* errors that operators care about.
#[async_trait]
pub trait ComponentBuilder {
    /// The value produced by the builder — usually an `Arc<dyn …>`
    /// around a client or resource pool.
    type Output;

    /// The configuration struct the builder consumes; must itself
    /// implement [`ComponentConfig`].
    type Config: ComponentConfig;

    /// Consumes a validated configuration and returns the live
    /// component.
    ///
    /// Implementations should treat `config` as already-validated:
    /// business-rule checks belong in [`ComponentConfig::validate`], not
    /// here. `build` is for IO and resource acquisition.
    async fn build(config: Self::Config) -> anyhow::Result<Self::Output>;
}
