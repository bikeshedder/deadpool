use std::fmt;

use deadpool::{managed::BuildError, Runtime};

use crate::{Pool, PoolConfig, RedisResult};

/// Configuration object. By enabling the `config` feature you can
/// read the configuration using the [`config`](https://crates.io/crates/config)
/// crate.
/// ## Example environment
/// ```env
/// REDIS__CONNECTION__ADDR=redis.example.com
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

/// This is a 1:1 copy of the `redis::ConnectionAddr` enumeration.
/// This is duplicated here in order to add support for the
/// `serde::Deserialize` trait which is required for the `config`
/// support.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub enum ConnectionAddr {
    Tcp(String, u16),
    TcpTls {
        host: String,
        port: u16,
        insecure: bool,
    },
    Unix(std::path::PathBuf),
}

impl Default for ConnectionAddr {
    fn default() -> Self {
        Self::Tcp("127.0.0.1".to_string(), 6379)
    }
}

impl From<ConnectionAddr> for redis::ConnectionAddr {
    fn from(addr: ConnectionAddr) -> Self {
        match addr {
            ConnectionAddr::Tcp(host, port) => Self::Tcp(host, port),
            ConnectionAddr::TcpTls {
                host,
                port,
                insecure,
            } => Self::TcpTls {
                host,
                port,
                insecure,
            },
            ConnectionAddr::Unix(path) => Self::Unix(path),
        }
    }
}

impl From<redis::ConnectionAddr> for ConnectionAddr {
    fn from(addr: redis::ConnectionAddr) -> Self {
        match addr {
            redis::ConnectionAddr::Tcp(host, port) => Self::Tcp(host, port),
            redis::ConnectionAddr::TcpTls {
                host,
                port,
                insecure,
            } => ConnectionAddr::TcpTls {
                host,
                port,
                insecure,
            },
            redis::ConnectionAddr::Unix(path) => Self::Unix(path),
        }
    }
}

/// This is a 1:1 copy of the `redis::ConnectionInfo` struct.
/// This is duplicated here in order to add support for the
/// `serde::Deserialize` trait which is required for the `config`
/// support.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct ConnectionInfo {
    addr: ConnectionAddr,
    #[serde(flatten)]
    redis: RedisConnectionInfo,
}

impl From<ConnectionInfo> for redis::ConnectionInfo {
    fn from(info: ConnectionInfo) -> Self {
        Self {
            addr: info.addr.into(),
            redis: info.redis.into(),
        }
    }
}

impl From<redis::ConnectionInfo> for ConnectionInfo {
    fn from(info: redis::ConnectionInfo) -> Self {
        Self {
            addr: info.addr.into(),
            redis: info.redis.into(),
        }
    }
}

impl redis::IntoConnectionInfo for ConnectionInfo {
    fn into_connection_info(self) -> RedisResult<redis::ConnectionInfo> {
        Ok(self.into())
    }
}

/// This is a 1:1 copy of the `redis::RedisConnectionInfo` struct.
/// This is duplicated here in order to add support for the
/// `serde::Deserialize` trait which is required for the `config`
/// support.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct RedisConnectionInfo {
    pub db: i64,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl From<RedisConnectionInfo> for redis::RedisConnectionInfo {
    fn from(info: RedisConnectionInfo) -> Self {
        Self {
            db: info.db,
            username: info.username,
            password: info.password,
        }
    }
}

impl From<redis::RedisConnectionInfo> for RedisConnectionInfo {
    fn from(info: redis::RedisConnectionInfo) -> Self {
        Self {
            db: info.db,
            username: info.username,
            password: info.password,
        }
    }
}

/// An error returned when pool creation fails.
#[derive(Debug)]
pub enum CreatePoolError {
    /// The pool configuration contained invalid options.
    Config(String),
    /// Redis returned an error while creating the pool.
    Redis(redis::RedisError),
}

impl From<redis::RedisError> for CreatePoolError {
    fn from(error: redis::RedisError) -> Self {
        Self::Redis(error)
    }
}

impl fmt::Display for CreatePoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(msg) => write!(f, "Config error: {}", msg),
            Self::Redis(error) => write!(f, "Config error: {}", error),
        }
    }
}

impl std::error::Error for CreatePoolError {}

/// Configuration object. By enabling the `config` feature you can
/// read the configuration using the [`config`](https://crates.io/crates/config)
/// crate.
///
/// ## Example environment
/// ```env
/// REDIS__ADDR=redis.example.com
/// PG__USER=john_doe
/// PG__PASSWORD=topsecret
/// PG__DBNAME=example
/// PG__POOL__MAX_SIZE=16
/// PG__POOL__TIMEOUTS__WAIT__SECS=5
/// PG__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
///
/// ## Example usage
/// ```rust,ignore
/// struct Config {
///     redis: deadpool_redis::Config,
/// }
/// impl Config {
///     pub fn from_env() -> Result<Self, ConfigError> {
///         let mut cfg = config::Config::new();
///         cfg.merge(config::Environment::new().separator("__")).unwrap();
///         cfg.try_into().unwrap()
///     }
/// }
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct Config {
    /// Redis URL<br>
    /// See [Connection Parameters](redis#connection-parameters)
    pub url: Option<String>,
    /// Redis ConnectionInfo structure
    pub connection: Option<ConnectionInfo>,
    /// Pool configuration
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Create pool using the current configuration
    pub fn create_pool(&self, runtime: Runtime) -> Result<Pool, BuildError<redis::RedisError>> {
        let manager = match (&self.url, &self.connection) {
            (Some(url), None) => crate::Manager::new(url.as_str())?,
            (None, Some(connection)) => crate::Manager::new(connection.clone())?,
            (None, None) => crate::Manager::new(ConnectionInfo::default())?,
            (Some(_), Some(_)) => {
                return Err(BuildError::Config(
                    "url and connection must not be specified at the same time.".to_owned(),
                ))
            }
        };
        let pool_config = self.get_pool_config();
        Pool::builder(manager)
            .config(pool_config)
            .runtime(runtime)
            .build()
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
            connection: Some(ConnectionInfo::default()),
            pool: None,
        }
    }
}
