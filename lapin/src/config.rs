#[cfg(feature = "config")]
use ::config_crate::{ConfigError, Environment};
use deadpool::managed::PoolConfig;

use crate::Pool;

/// Configuration object. By enabling the `config` feature you can
/// read the configuration using the [`config`](https://crates.io/crates/config)
/// crate. e.g.:
#[derive(Debug)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct Config {
    url: Option<String>,
    pool: Option<PoolConfig>,
}

impl Config {
    /// Create configuration from environment variables.
    #[cfg(feature = "config")]
    pub fn from_env(prefix: &str) -> Result<Self, ConfigError> {
        let mut cfg = ::config_crate::Config::new();
        cfg.merge(Environment::with_prefix(prefix))?;
        cfg.try_into()
    }
    /// Create pool using the current configuration
    pub fn create_pool(&self) -> Pool {
        let url = self.get_url().to_string();
        let manager = crate::Manager::new(url, lapin::ConnectionProperties::default());
        let pool_config = self.get_pool_config();
        Pool::from_config(manager, pool_config)
    }
    /// Get `URL` which can be used to connect to
    /// the database.
    pub fn get_url(&self) -> &str {
        if let Some(url) = &self.url {
            url
        } else {
            "amqp://127.0.0.1:5672/%2f"
        }
    }
    /// Get `deadpool::PoolConfig` which can be used to construct a
    /// `deadpool::managed::Pool` instance.
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.clone().unwrap_or_default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: None,
            pool: None,
        }
    }
}
