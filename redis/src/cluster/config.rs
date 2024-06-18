pub use crate::config::ConfigError;
use crate::ConnectionInfo;

use super::{CreatePoolError, Pool, PoolBuilder, PoolConfig, Runtime};

/// Configuration object.
///
/// # Example (from environment)
///
/// By enabling the `serde` feature you can read the configuration using the
/// [`config`](https://crates.io/crates/config) crate as following:
/// ```env
/// REDIS_CLUSTER__URLS=redis://127.0.0.1:7000,redis://127.0.0.1:7001
/// REDIS_CLUSTER__POOL__MAX_SIZE=16
/// REDIS_CLUSTER__POOL__TIMEOUTS__WAIT__SECS=2
/// REDIS_CLUSTER__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ```rust
/// #[derive(serde::Deserialize)]
/// struct Config {
///     redis_cluster: deadpool_redis::cluster::Config,
/// }
///
/// impl Config {
///     pub fn from_env() -> Result<Self, config::ConfigError> {
///         let mut cfg = config::Config::builder()
///            .add_source(
///                config::Environment::default()
///                .separator("__")
///                .try_parsing(true)
///                .list_separator(","),
///            )
///            .build()?;
///            cfg.try_deserialize()
///     }
/// }
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Config {
    /// Redis URLs.
    ///
    /// See [Connection Parameters](redis#connection-parameters).
    pub urls: Option<Vec<String>>,

    /// [`redis::ConnectionInfo`] structures.
    pub connections: Option<Vec<ConnectionInfo>>,

    /// Pool configuration.
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    pub fn create_pool(&self, runtime: Option<Runtime>) -> Result<Pool, CreatePoolError> {
        let mut builder = self.builder().map_err(CreatePoolError::Config)?;
        if let Some(runtime) = runtime {
            builder = builder.runtime(runtime);
        }
        builder.build().map_err(CreatePoolError::Build)
    }

    /// Creates a new [`PoolBuilder`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`ConfigError`] for details.
    pub fn builder(&self) -> Result<PoolBuilder, ConfigError> {
        let manager = match (&self.urls, &self.connections) {
            (Some(urls), None) => {
                super::Manager::new(urls.iter().map(|url| url.as_str()).collect())?
            }
            (None, Some(connections)) => super::Manager::new(connections.clone())?,
            (None, None) => super::Manager::new(vec![ConnectionInfo::default()])?,
            (Some(_), Some(_)) => return Err(ConfigError::UrlAndConnectionSpecified),
        };
        let pool_config = self.get_pool_config();
        Ok(Pool::builder(manager).config(pool_config))
    }

    /// Returns [`deadpool::managed::PoolConfig`] which can be used to construct
    /// a [`deadpool::managed::Pool`] instance.
    #[must_use]
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.unwrap_or_default()
    }

    /// Creates a new [`Config`] from the given Redis URL (like
    /// `redis://127.0.0.1`).
    #[must_use]
    pub fn from_urls<T: Into<Vec<String>>>(urls: T) -> Config {
        Config {
            urls: Some(urls.into()),
            connections: None,
            pool: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            urls: None,
            connections: Some(vec![ConnectionInfo::default()]),
            pool: None,
        }
    }
}
