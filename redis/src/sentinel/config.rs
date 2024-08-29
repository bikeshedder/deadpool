use redis::sentinel::SentinelNodeConnectionInfo;
use redis::TlsMode;

pub use crate::config::ConfigError;
use crate::{ConnectionAddr, ConnectionInfo};

use super::{CreatePoolError, Pool, PoolBuilder, PoolConfig, Runtime};

/// Configuration object.
///
/// # Example (from environment)
///
/// By enabling the `serde` feature you can read the configuration using the
/// [`config`](https://crates.io/crates/config) crate as following:
/// ```env
/// REDIS_SENTINEL__URLS=redis://127.0.0.1:26379,redis://127.0.0.1:26380
/// REDIS_SENTINEL__MASTER_NAME=mymaster
/// REDIS_SENTINEL__SERVER_TYPE=master
/// REDIS_SENTINEL__POOL__MAX_SIZE=16
/// REDIS_SENTINEL__POOL__TIMEOUTS__WAIT__SECS=2
/// REDIS_SENTINEL__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ```rust
/// #[derive(serde::Deserialize)]
/// struct Config {
///     redis_sentinel: deadpool_redis::sentinel::Config,
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
    /// ServerType
    ///
    /// [`SentinelServerType`]
    #[serde(default)]
    pub server_type: SentinelServerType,
    /// Sentinel setup master name. default value is `mymaster`
    #[serde(default = "default_master_name")]
    pub master_name: String,
    /// [`redis::ConnectionInfo`] structures.
    pub connections: Option<Vec<ConnectionInfo>>,
    // SentinelNodeConnectionInfo doesn't implement debug, so we can't
    // use it as a field, also they have identical fields.
    sentinel_connection_info: Option<ConnectionInfo>,
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
        let sentinel_node_connection_info = self.sentinel_connection_info.clone().map(|c| {
            let tls_mode = match c.addr {
                ConnectionAddr::TcpTls { insecure: i, .. } => {
                    if i {
                        Some(TlsMode::Insecure)
                    } else {
                        Some(TlsMode::Secure)
                    }
                }
                ConnectionAddr::Unix(_) | ConnectionAddr::Tcp(_, _) => None,
            };

            SentinelNodeConnectionInfo {
                tls_mode,
                redis_connection_info: Some(c.redis.into()),
            }
        });

        let manager = match (&self.urls, &self.connections) {
            (Some(urls), None) => super::Manager::new(
                urls.iter().map(|url| url.as_str()).collect(),
                self.master_name.clone(),
                sentinel_node_connection_info,
                self.server_type,
            )?,
            (None, Some(connections)) => super::Manager::new(
                connections.clone(),
                self.master_name.clone(),
                sentinel_node_connection_info,
                self.server_type,
            )?,
            (None, None) => super::Manager::new(
                vec![ConnectionInfo::default()],
                self.master_name.clone(),
                sentinel_node_connection_info,
                self.server_type,
            )?,
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
    pub fn from_urls<T: Into<Vec<String>>>(
        urls: T,
        master_name: String,
        server_type: SentinelServerType,
    ) -> Config {
        Config {
            urls: Some(urls.into()),
            connections: None,
            master_name,
            server_type,
            pool: None,
            sentinel_connection_info: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let default_connection_info = ConnectionInfo {
            addr: ConnectionAddr::Tcp("127.0.0.1".to_string(), 26379),
            ..ConnectionInfo::default()
        };

        Self {
            urls: None,
            connections: Some(vec![default_connection_info.clone()]),
            server_type: SentinelServerType::Master,
            master_name: String::from("mymaster"),
            pool: None,
            sentinel_connection_info: Some(default_connection_info),
        }
    }
}

fn default_master_name() -> String {
    "mymaster".to_string()
}

/// This type is a wrapper for [`redis::sentinel::SentinelServerType`] for serialize/deserialize.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum SentinelServerType {
    #[default]
    /// Master connections only
    Master,
    /// Replica connections only
    Replica,
}

impl From<redis::sentinel::SentinelServerType> for SentinelServerType {
    fn from(value: redis::sentinel::SentinelServerType) -> Self {
        match value {
            redis::sentinel::SentinelServerType::Master => SentinelServerType::Master,
            redis::sentinel::SentinelServerType::Replica => SentinelServerType::Replica,
        }
    }
}

impl From<SentinelServerType> for redis::sentinel::SentinelServerType {
    fn from(value: SentinelServerType) -> Self {
        match value {
            SentinelServerType::Master => redis::sentinel::SentinelServerType::Master,
            SentinelServerType::Replica => redis::sentinel::SentinelServerType::Replica,
        }
    }
}
