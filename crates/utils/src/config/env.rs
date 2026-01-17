use config::{Config, Environment};
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Debug, Deserialize)]
pub struct EnvConfig {
    pub rust_log: Option<String>,
}

impl EnvConfig {
    fn new() -> Result<Self, config::ConfigError> {
        let s = Config::builder()
            .add_source(Environment::default())
            .build()?;
        // Deserialize into our struct
        s.try_deserialize()
    }
}

/// Global environment configuration
pub static ENV_SETTINGS: LazyLock<EnvConfig> =
    LazyLock::new(|| {
        dotenvy::dotenv().ok();
        EnvConfig::new().expect(
            "Failed to load configuration from environment",
        )
    });

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = &ENV_SETTINGS;
        dbg!(&config.rust_log);
    }
}
