use deadpool::Runtime;
#[cfg(feature = "rt_tokio_1")]
use tokio_amqp::LapinTokioExt;

#[cfg(feature = "rt_async-std_1")]
use async_amqp::LapinAsyncStdExt;

use crate::{Pool, PoolConfig};

pub type BuildError = deadpool::managed::BuildError<lapin::Error>;

/// Configuration object. By enabling the `config` feature you can
/// read the configuration using the [`config`](https://crates.io/crates/config)
/// crate. e.g.:
///
/// ## Example environment
/// ```env
/// AMQP__URL=amqp://127.0.0.1:5672/%2f
/// AMQP__POOL__MAX_SIZE=16
/// AMQP__POOL__TIMEOUTS__WAIT__SECS=2
/// AMQP__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ## Example usage
/// ```rust,ignore
/// struct Config {
///     amqp: deadpool_lapin::Config,
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
    /// AMQP server URL
    pub url: Option<String>,
    /// Pool configuration
    pub pool: Option<PoolConfig>,
    /// Connection properties
    #[cfg_attr(feature = "config", serde(skip))]
    pub connection_properties: lapin::ConnectionProperties,
}

impl Config {
    /// Create pool using the current configuration
    pub fn create_pool(&self, runtime: Runtime) -> Result<Pool, BuildError> {
        let url = self.get_url().to_string();
        let pool_config = self.get_pool_config();
        let connection_properties = self.connection_properties.clone();
        let connection_properties = match runtime {
            #[cfg(feature = "rt_tokio_1")]
            Runtime::Tokio1 => connection_properties.with_tokio(),
            #[cfg(feature = "rt_async-std_1")]
            Runtime::AsyncStd1 => connection_properties.with_async_std(),
        };
        let manager = crate::Manager::new(url, connection_properties);
        Pool::builder(manager)
            .config(pool_config)
            .runtime(runtime)
            .build()
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
            connection_properties: lapin::ConnectionProperties::default(),
        }
    }
}
