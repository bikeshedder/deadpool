#[cfg(feature = "config")]
use ::config_crate::{ConfigError, Environment};
use deadpool::managed::PoolConfig;

use crate::{Pool, RedisResult};

/// Configuration object. By enabling the `config` feature you can
/// read the configuration using the [`config`](https://crates.io/crates/config)
/// crate.
/// ## Example environment
/// ```env
/// REDIS__URL=pg.example.com
/// REDIS__POOL__MAX_SIZE=16
/// REDIS__POOL__TIMEOUTS__WAIT__SECS=2
/// REDIS__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ## Example usage
/// ```rust,ignore
/// struct Config {
///     redis: deadpool_postgres::Config,
/// }
/// impl Config {
///     pub fn from_env() -> Result<Self, config::ConfigError> {
///         let mut cfg = config::Config::new();
///         cfg.merge(config::Environment::new().separator("__")).unwrap();
///         cfg.try_into().unwrap()
///     }
/// }
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct Config {
    /// Redis URL
    /// See https://docs.rs/redis/0.15.1/redis/#connection-parameters
    pub url: Option<String>,
    /// Pool configuration
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Create configuration from environment variables.
    #[cfg(feature = "config")]
    #[deprecated(
        since = "0.7.0",
        note = "Please embed this structure in your own config structure and use `config::Config` directly."
    )]
    pub fn from_env(prefix: &str) -> Result<Self, ConfigError> {
        let mut cfg = ::config_crate::Config::new();
        cfg.merge(Environment::with_prefix(prefix))?;
        cfg.try_into()
    }
    /// Create pool using the current configuration
    pub fn create_pool(&self) -> RedisResult<Pool> {
        let url = self.get_url();
        let manager = crate::Manager::new(url)?;
        let pool_config = self.get_pool_config();
        Ok(Pool::from_config(manager, pool_config))
    }
    /// Get `URL` which can be used to connect to
    /// the database.
    pub fn get_url(&self) -> &str {
        if let Some(url) = &self.url {
            url
        } else {
            "redis://127.0.0.1/"
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
