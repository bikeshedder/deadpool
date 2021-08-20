#[cfg(feature = "rt_async-std_1")]
use async_amqp::LapinAsyncStdExt;
use deadpool::{managed, Runtime};
#[cfg(feature = "rt_tokio_1")]
use tokio_amqp::LapinTokioExt;

use crate::{Manager, Pool, PoolConfig};

/// Error of building a [`Pool`].
pub type BuildError = managed::BuildError<lapin::Error>;

/// Configuration object.
///
/// # Example (from environment)
///
/// By enabling the `serde` feature you can read the configuration using the
/// [`config`](https://crates.io/crates/config) crate as following:
/// ```env
/// AMQP__URL=amqp://127.0.0.1:5672/%2f
/// AMQP__POOL__MAX_SIZE=16
/// AMQP__POOL__TIMEOUTS__WAIT__SECS=2
/// AMQP__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ```rust
/// # #[derive(serde_1::Deserialize)]
/// # #[serde(crate = "serde_1")]
/// struct Config {
///     amqp: deadpool_lapin::Config,
/// }
///
/// impl Config {
///     pub fn from_env() -> Result<Self, config::ConfigError> {
///         let mut cfg = config::Config::new();
///         cfg.merge(config::Environment::new().separator("__")).unwrap();
///         cfg.try_into()
///     }
/// }
/// ```
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde_1::Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde_1"))]
pub struct Config {
    /// AMQP server URL.
    pub url: Option<String>,

    /// [`Pool`] configuration.
    pub pool: Option<PoolConfig>,

    /// Connection properties.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub connection_properties: lapin::ConnectionProperties,
}

impl Config {
    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`BuildError`] and [`lapin::Error`] for details.
    ///
    /// [`BuildError`]: managed::BuildError
    pub fn create_pool(&self, runtime: Runtime) -> Result<Pool, BuildError> {
        let url = self.get_url().to_string();
        let pool_config = self.get_pool_config();

        let conn_props = self.connection_properties.clone();
        let conn_props = match runtime {
            #[cfg(feature = "rt_tokio_1")]
            Runtime::Tokio1 => conn_props.with_tokio(),
            #[cfg(feature = "rt_async-std_1")]
            Runtime::AsyncStd1 => conn_props.with_async_std(),
        };

        Pool::builder(Manager::new(url, conn_props))
            .config(pool_config)
            .runtime(runtime)
            .build()
    }

    /// Returns URL which can be used to connect to the database.
    pub fn get_url(&self) -> &str {
        if let Some(url) = &self.url {
            url
        } else {
            "amqp://127.0.0.1:5672/%2f"
        }
    }

    /// Returns [`deadpool::managed::PoolConfig`] which can be used to construct
    /// a [`deadpool::managed::Pool`] instance.
    #[must_use]
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.clone().unwrap_or_default()
    }
}
