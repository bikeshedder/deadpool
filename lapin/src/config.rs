use std::convert::Infallible;

#[cfg(feature = "rt_async-std_1")]
use async_amqp::LapinAsyncStdExt as _;
#[cfg(feature = "rt_tokio_1")]
use tokio_amqp::LapinTokioExt as _;

use crate::{CreatePoolError, Manager, Pool, PoolBuilder, PoolConfig, Runtime};

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
/// # use serde_1 as serde;
/// #
/// #[derive(serde::Deserialize)]
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
#[cfg_attr(feature = "serde", derive(serde_1::Deserialize, serde_1::Serialize))]
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
    /// See [`CreatePoolError`] for details.
    pub fn create_pool(&self, runtime: Option<Runtime>) -> Result<Pool, CreatePoolError> {
        self.builder(runtime)
            .build()
            .map_err(CreatePoolError::Build)
    }

    /// Creates a new [`PoolBuilder`] using this [`Config`].
    pub fn builder(&self, runtime: Option<Runtime>) -> PoolBuilder {
        let url = self.get_url().to_string();
        let pool_config = self.get_pool_config();

        let conn_props = self.connection_properties.clone();
        let conn_props = match runtime {
            None => conn_props,
            #[cfg(feature = "rt_tokio_1")]
            Some(Runtime::Tokio1) => conn_props.with_tokio(),
            #[cfg(feature = "rt_async-std_1")]
            Some(Runtime::AsyncStd1) => conn_props.with_async_std(),
        };

        let mut builder = Pool::builder(Manager::new(url, conn_props)).config(pool_config);

        if let Some(runtime) = runtime {
            builder = builder.runtime(runtime)
        }

        builder
    }

    /// Returns URL which can be used to connect to the database.
    pub fn get_url(&self) -> &str {
        self.url.as_deref().unwrap_or("amqp://127.0.0.1:5672/%2f")
    }

    /// Returns [`deadpool::managed::PoolConfig`] which can be used to construct
    /// a [`deadpool::managed::Pool`] instance.
    #[must_use]
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.unwrap_or_default()
    }
}

/// This error is returned if there is something wrong with the lapin configuration.
///
/// This is just a type alias to [`Infallible`] at the moment as there
/// is no validation happening at the configuration phase.
pub type ConfigError = Infallible;
