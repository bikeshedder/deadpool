use std::fmt;

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

impl Into<redis::ConnectionAddr> for ConnectionAddr {
    fn into(self) -> redis::ConnectionAddr {
        match self {
            Self::Tcp(host, port) => redis::ConnectionAddr::Tcp(host, port),
            Self::TcpTls {
                host,
                port,
                insecure,
            } => redis::ConnectionAddr::TcpTls {
                host,
                port,
                insecure,
            },
            Self::Unix(path) => redis::ConnectionAddr::Unix(path),
        }
    }
}

impl Into<ConnectionAddr> for redis::ConnectionAddr {
    fn into(self) -> ConnectionAddr {
        match self {
            redis::ConnectionAddr::Tcp(host, port) => ConnectionAddr::Tcp(host, port),
            redis::ConnectionAddr::TcpTls {
                host,
                port,
                insecure,
            } => ConnectionAddr::TcpTls {
                host,
                port,
                insecure,
            },
            redis::ConnectionAddr::Unix(path) => ConnectionAddr::Unix(path),
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
    addr: Box<ConnectionAddr>,
    db: i64,
    username: Option<String>,
    passwd: Option<String>,
}

impl Into<redis::ConnectionInfo> for ConnectionInfo {
    fn into(self) -> redis::ConnectionInfo {
        redis::ConnectionInfo {
            addr: Box::new((*self.addr).into()),
            db: self.db,
            username: self.username,
            passwd: self.passwd,
        }
    }
}

impl Into<ConnectionInfo> for redis::ConnectionInfo {
    fn into(self) -> ConnectionInfo {
        ConnectionInfo {
            addr: Box::new((*self.addr).into()),
            db: self.db,
            username: self.username,
            passwd: self.passwd,
        }
    }
}

impl redis::IntoConnectionInfo for ConnectionInfo {
    fn into_connection_info(self) -> RedisResult<redis::ConnectionInfo> {
        Ok(self.into())
    }
}

#[derive(Debug)]
pub enum CreatePoolError {
    Config(String),
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
    /// Redis URL
    /// See https://docs.rs/redis/0.20.0/redis/index.html#connection-parameters
    pub url: Option<String>,
    /// Redis ConnectionInfo structure
    pub connection: Option<ConnectionInfo>,
    /// Pool configuration
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Create pool using the current configuration
    pub fn create_pool(&self) -> Result<Pool, CreatePoolError> {
        let manager = match (&self.url, &self.connection) {
            (Some(url), None) => crate::Manager::new(url.as_str())?,
            (None, Some(connection)) => crate::Manager::new(connection.clone())?,
            (None, None) => crate::Manager::new(ConnectionInfo::default())?,
            (Some(_), Some(_)) => {
                return Err(CreatePoolError::Config(
                    "url and connection must not be specified at the same time.".to_owned(),
                ))
            }
        };
        let pool_config = self.get_pool_config();
        Ok(Pool::from_config(manager, pool_config))
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
