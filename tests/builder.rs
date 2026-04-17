//! Integration test for [`ComponentBuilder`].
//!
//! The trait is `async`, so we poll it through a minimal executor —
//! we deliberately avoid a full tokio runtime to keep dev-deps small.

use hosanna_rs_config::{ComponentBuilder, ComponentConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DummyConfig;

impl ComponentConfig for DummyConfig {
    fn env_prefix() -> &'static str {
        "BUILDER_DUMMY"
    }
}

struct Counter;

#[async_trait::async_trait]
impl ComponentBuilder for Counter {
    type Output = u32;
    type Config = DummyConfig;

    async fn build(_config: Self::Config) -> anyhow::Result<Self::Output> {
        Ok(42)
    }
}

#[test]
fn component_builder_returns_output() {
    let value = futures_executor::block_on(Counter::build(DummyConfig));
    assert_eq!(value.expect("build must succeed"), 42);
}
