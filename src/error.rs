//! Error types surfaced by the `hosanna-rs-config` crate.
//!
//! The loader pipeline has two failure modes and the error type reflects
//! that precisely: a [`ConfigError::Load`] wraps anything the underlying
//! [`config`] crate reports (missing required field, malformed JSON,
//! unparseable environment variable), while [`ConfigError::Validation`]
//! carries a business-rule rejection returned by the component's own
//! [`ComponentConfig::validate`] implementation.
//!
//! Every variant carries the `component` label — the value of
//! [`ComponentConfig::env_prefix`] — so a single `ConfigError` in a log
//! line tells the operator which component's configuration is at fault
//! without having to consult the stack trace.
//!
//! [`ComponentConfig`]: crate::traits::ComponentConfig
//! [`ComponentConfig::validate`]: crate::traits::ComponentConfig::validate
//! [`ComponentConfig::env_prefix`]: crate::traits::ComponentConfig::env_prefix

use thiserror::Error;

/// Errors that can arise while loading or validating a component's
/// configuration.
///
/// Both variants are constructed only by [`ComponentConfigLoader::load`];
/// downstream code typically just propagates them with `?`.
///
/// [`ComponentConfigLoader::load`]: crate::types::ComponentConfigLoader::load
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The [`config`] crate failed to assemble the final configuration —
    /// this covers missing required keys, malformed JSON, unparseable
    /// environment variables, and any other IO or deserialization failure
    /// that happens *before* the validation step.
    #[error("failed to load config for component '{component}': {source}")]
    Load {
        /// The component's env prefix, copied from
        /// [`ComponentConfig::env_prefix`]. Stable identifier useful for
        /// routing logs and alerts.
        ///
        /// [`ComponentConfig::env_prefix`]: crate::traits::ComponentConfig::env_prefix
        component: &'static str,
        /// The underlying error from the [`config`] crate.
        #[source]
        source: config::ConfigError,
    },

    /// The configuration deserialized cleanly but was rejected by the
    /// component's own [`ComponentConfig::validate`] implementation.
    ///
    /// The `reason` string is the verbatim message returned by
    /// `validate()`; it should be human-readable and safe to log.
    ///
    /// [`ComponentConfig::validate`]: crate::traits::ComponentConfig::validate
    #[error("validation failed for component '{component}': {reason}")]
    Validation {
        /// The component's env prefix, copied from
        /// [`ComponentConfig::env_prefix`].
        ///
        /// [`ComponentConfig::env_prefix`]: crate::traits::ComponentConfig::env_prefix
        component: &'static str,
        /// The human-readable rejection reason returned by
        /// [`ComponentConfig::validate`].
        ///
        /// [`ComponentConfig::validate`]: crate::traits::ComponentConfig::validate
        reason: String,
    },
}
