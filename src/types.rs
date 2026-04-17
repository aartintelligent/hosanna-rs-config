//! Concrete types exported by the crate.
//!
//! At present a single type lives here — [`ComponentConfigLoader`] — but
//! the module is named for its role (the concrete, instantiable side of
//! the crate) rather than its current inhabitants, so future additions
//! such as a non-JSON loader variant have an obvious home.

use std::marker::PhantomData;

use config::{Config, Environment, File, FileFormat};

use crate::{error::ConfigError, traits::ComponentConfig};

/// Builder that composes the configuration sources for a component and
/// returns a validated instance of its [`ComponentConfig`] struct.
///
/// The loader is intentionally stateless: [`load`](Self::load) consumes
/// `self`, so a loader is built, optionally customised, and then used
/// exactly once.
///
/// # Source priority
///
/// The final configuration is assembled from two layers. Later sources
/// override earlier ones, so the effective priority is:
///
/// 1. JSON file — lowest precedence; skipped silently if
///    [`ComponentConfig::config_file`] is `None` or the file does not
///    exist on disk.
/// 2. Environment variables — highest precedence; always applied.
///
/// This mirrors the 12-factor convention: the file is the committed
/// default, the environment is what the operator actually deploys.
///
/// # Usage
///
/// ```rust,no_run
/// use hosanna_rs_config::{ComponentConfig, ComponentConfigLoader};
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize)]
/// struct NatsConfig { url: String }
///
/// impl ComponentConfig for NatsConfig {
///     fn env_prefix() -> &'static str { "NATS" }
///     fn config_file() -> Option<&'static str> { Some("config/nats") }
/// }
///
/// let cfg = ComponentConfigLoader::<NatsConfig>::new().load()?;
/// # Ok::<_, hosanna_rs_config::ConfigError>(())
/// ```
pub struct ComponentConfigLoader<C: ComponentConfig> {
    env_prefix: &'static str,
    env_separator: &'static str,
    config_file: Option<&'static str>,
    _marker: PhantomData<fn() -> C>,
}

impl<C: ComponentConfig> ComponentConfigLoader<C> {
    /// Constructs a loader seeded from the defaults declared on the
    /// [`ComponentConfig`] implementation.
    ///
    /// Call the `with_*` methods to override any single field before
    /// calling [`load`](Self::load).
    pub fn new() -> Self {
        Self {
            env_prefix: C::env_prefix(),
            env_separator: C::env_separator(),
            config_file: C::config_file(),
            _marker: PhantomData,
        }
    }

    /// Overrides the environment-variable prefix for this load.
    ///
    /// Useful in tests and in multi-tenant binaries where two instances
    /// of the same component must be loaded from disjoint prefixes.
    pub fn with_env_prefix(mut self, prefix: &'static str) -> Self {
        self.env_prefix = prefix;
        self
    }

    /// Overrides the environment-variable separator for this load.
    pub fn with_env_separator(mut self, separator: &'static str) -> Self {
        self.env_separator = separator;
        self
    }

    /// Overrides the JSON configuration-file path for this load.
    ///
    /// The path is still treated as optional: if the file does not
    /// exist, the loader silently falls back to environment variables.
    pub fn with_config_file(mut self, path: &'static str) -> Self {
        self.config_file = Some(path);
        self
    }

    /// Assembles the configuration, deserialises it into `C`, and runs
    /// [`ComponentConfig::validate`] on the result.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Load`] if the [`config`] crate cannot
    /// build or deserialise the layered configuration (missing required
    /// field, bad JSON, unparseable env value, …), and
    /// [`ConfigError::Validation`] if the struct's own `validate()`
    /// rejects the final value.
    pub fn load(self) -> Result<C, ConfigError> {
        let mut builder = Config::builder();

        if let Some(path) = self.config_file {
            builder = builder.add_source(File::new(path, FileFormat::Json).required(false));
        }

        // `prefix_separator` governs the split between the prefix and the
        // first key segment (e.g. `NATS` / `URL`); `separator` governs
        // splits inside nested keys (e.g. `TLS` / `CERT_PATH`). The
        // crate convention is to use the same sequence for both so a
        // single knob — `env_separator()` — is enough to describe the
        // whole naming scheme.
        builder = builder.add_source(
            Environment::with_prefix(self.env_prefix)
                .prefix_separator(self.env_separator)
                .separator(self.env_separator)
                .try_parsing(true),
        );

        let config = builder
            .build()
            .and_then(|c| c.try_deserialize::<C>())
            .map_err(|source| ConfigError::Load {
                component: C::env_prefix(),
                source,
            })?;

        config
            .validate()
            .map_err(|reason| ConfigError::Validation {
                component: C::env_prefix(),
                reason,
            })?;

        Ok(config)
    }
}

impl<C: ComponentConfig> Default for ComponentConfigLoader<C> {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────
//
// The tests mutate process-wide environment variables, so each case uses
// a distinct prefix (`LOADER_*`) to avoid cross-contamination even under
// the `--test-threads=1` discipline enforced in CI.

#[cfg(test)]
#[allow(unsafe_code)] // edition 2024 requires `unsafe` around `std::env::set_var`.
mod tests {
    use serde::Deserialize;

    use super::ComponentConfigLoader;
    use crate::traits::ComponentConfig;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct SimpleConfig {
        url: String,
        port: u16,
    }

    impl ComponentConfig for SimpleConfig {
        fn env_prefix() -> &'static str {
            "LOADER_SIMPLE"
        }
    }

    #[test]
    fn loads_from_env_variables() {
        // SAFETY-ish: tests within this module serialise through distinct
        // prefixes; see the module-level comment for rationale.
        unsafe {
            std::env::set_var("LOADER_SIMPLE__URL", "http://localhost");
            std::env::set_var("LOADER_SIMPLE__PORT", "8080");
        }

        let config = ComponentConfigLoader::<SimpleConfig>::new()
            .load()
            .expect("should load from env");

        assert_eq!(config.url, "http://localhost");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn missing_json_file_is_not_blocking() {
        unsafe {
            std::env::set_var("LOADER_MISSING__URL", "http://localhost");
            std::env::set_var("LOADER_MISSING__PORT", "9090");
        }

        #[derive(Debug, Deserialize)]
        struct MissingFileConfig {
            url: String,
            port: u16,
        }

        impl ComponentConfig for MissingFileConfig {
            fn env_prefix() -> &'static str {
                "LOADER_MISSING"
            }
        }

        let config = ComponentConfigLoader::<MissingFileConfig>::new()
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

        let config = ComponentConfigLoader::<SimpleConfig>::new()
            .with_env_prefix("LOADER_OVERRIDE")
            .load()
            .expect("should load with overridden prefix");

        assert_eq!(config.url, "http://override");
        assert_eq!(config.port, 1234);
    }

    #[test]
    fn validation_error_is_propagated() {
        #[derive(Debug, Deserialize)]
        struct StrictConfig {
            port: u16,
        }

        impl ComponentConfig for StrictConfig {
            fn env_prefix() -> &'static str {
                "LOADER_STRICT"
            }

            fn validate(&self) -> Result<(), String> {
                if self.port < 1024 {
                    return Err(format!("port {} is reserved", self.port));
                }
                Ok(())
            }
        }

        unsafe {
            std::env::set_var("LOADER_STRICT__PORT", "80");
        }

        let err = ComponentConfigLoader::<StrictConfig>::new()
            .load()
            .expect_err("validation must reject a reserved port");

        match err {
            crate::error::ConfigError::Validation { component, reason } => {
                assert_eq!(component, "LOADER_STRICT");
                assert!(reason.contains("80"));
            }
            other => panic!("expected Validation error, got {other:?}"),
        }
    }
}
