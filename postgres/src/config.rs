//! This module describes configuration used for [`Pool`] creation.

use std::env;
use std::fmt;
use std::time::Duration;

use tokio_postgres::config::{
    ChannelBinding as PgChannelBinding, SslMode as PgSslMode,
    TargetSessionAttrs as PgTargetSessionAttrs,
};
use tokio_postgres::tls::{MakeTlsConnect, TlsConnect};
use tokio_postgres::Socket;

use crate::{Pool, PoolConfig};

/// An error which is returned by `Config::create_pool` if something is
/// wrong with the configuration.
#[derive(Debug)]
pub enum ConfigError {
    /// Message of the error.
    Message(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => write!(f, "{}", message),
        }
    }
}

#[cfg(feature = "config")]
impl Into<::config_crate::ConfigError> for ConfigError {
    fn into(self) -> ::config_crate::ConfigError {
        match self {
            Self::Message(message) => ::config_crate::ConfigError::Message(message),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Properties required of a session.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
#[non_exhaustive]
pub enum TargetSessionAttrs {
    /// No special properties are required.
    Any,
    /// The session must allow writes.
    ReadWrite,
}

impl Into<PgTargetSessionAttrs> for TargetSessionAttrs {
    fn into(self) -> PgTargetSessionAttrs {
        match self {
            Self::Any => PgTargetSessionAttrs::Any,
            Self::ReadWrite => PgTargetSessionAttrs::ReadWrite,
        }
    }
}

/// TLS configuration.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
#[non_exhaustive]
pub enum SslMode {
    /// Do not use TLS.
    Disable,
    /// Attempt to connect with TLS but allow sessions without.
    Prefer,
    /// Require the use of TLS.
    Require,
}

impl Into<PgSslMode> for SslMode {
    fn into(self) -> PgSslMode {
        match self {
            Self::Disable => PgSslMode::Disable,
            Self::Prefer => PgSslMode::Prefer,
            Self::Require => PgSslMode::Require,
        }
    }
}

/// Channel binding configuration.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
#[non_exhaustive]
pub enum ChannelBinding {
    /// Do not use channel binding.
    Disable,
    /// Attempt to use channel binding but allow sessions without.
    Prefer,
    /// Require the use of channel binding.
    Require,
}

impl Into<PgChannelBinding> for ChannelBinding {
    fn into(self) -> PgChannelBinding {
        match self {
            Self::Disable => PgChannelBinding::Disable,
            Self::Prefer => PgChannelBinding::Prefer,
            Self::Require => PgChannelBinding::Require,
        }
    }
}

/// This enum is used to control how the connection is recycled.
/// **Attention:** The current default is `Verified` but will be changed
/// to `Fast` in the next minor release of `deadpool-postgres`. Please
/// make sure to explicitly state this if you want to keep using the
/// `Verified` recycling method.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub enum RecyclingMethod {
    /// Only run `Client::is_closed` when recycling existing connections.
    /// Unless you have special needs this is a safe choice.
    Fast,
    /// Run `Client::is_closed` and execute a test query. This is slower
    /// but guarantees that the database connection is ready to be used.
    /// Normally `Client::is_closed` should be enough to filter out bad
    /// connections but under some circumstances (i.e. hard-closed
    /// network connections) it is possible that `Client::is_closed` returns
    /// `false` but the connection is dead. You will receive an error on
    /// your first query then.
    Verified,
    /// Like `Verified` query method but instead use the following sequence of
    /// statements which guarantees a prestine connection:
    ///
    /// ```rust,ignore
    /// CLOSE ALL;
    /// SET SESSION AUTHORIZATION DEFAULT;
    /// RESET ALL;
    /// UNLISTEN *;
    /// SELECT pg_advisory_unlock_all();
    /// DISCARD TEMP;
    /// DISCARD SEQUENCES;
    /// ```
    ///
    /// This is similar to calling `DISCARD ALL` but does not call
    /// `DEALLOCATE ALL` and `DISCARD PLAN` so that the statement cache
    /// is not rendered ineffective.
    Clean,
    /// Like `Verified` but allows to specify a custom SQL to be executed.
    Custom(String),
}

const DISCARD_SQL: &str = "
CLOSE ALL;
SET SESSION AUTHORIZATION DEFAULT;
RESET ALL;
UNLISTEN *;
SELECT pg_advisory_unlock_all();
DISCARD TEMP;
DISCARD SEQUENCES;
";

impl RecyclingMethod {
    /// Return SQL query to be executed when recycling a connection.
    pub fn query(&self) -> Option<&str> {
        match self {
            Self::Fast => None,
            Self::Verified => Some(""),
            Self::Clean => Some(DISCARD_SQL),
            Self::Custom(sql) => Some(&sql),
        }
    }
}

impl Default for RecyclingMethod {
    fn default() -> Self {
        Self::Verified
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
/// Configuration object for the manager. This currently only makes it
/// possible to specify which recycling method should be used when retrieving
/// existing objects from the pool.
pub struct ManagerConfig {
    /// This controls how the connection is recycled. See `RecyclingMethod`
    pub recycling_method: RecyclingMethod,
}

/// Configuration object. By enabling the `config` feature you can
/// read the configuration using the [`config`](https://crates.io/crates/config)
/// crate.
///
/// ## Example environment
/// ```env
/// PG__HOST=pg.example.com
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
///     pg: deadpool_postgres::Config,
/// }
/// impl Config {
///     pub fn from_env() -> Result<Self, ConfigError> {
///         let mut cfg = config::Config::new();
///         cfg.merge(config::Environment::new().separator("__")).unwrap();
///         cfg.try_into().unwrap()
///     }
/// }
/// ```
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct Config {
    /// See `tokio_postgres::Config::user`
    pub user: Option<String>,
    /// See `tokio_postgres::Config::password`
    pub password: Option<String>,
    /// See `tokio_postgres::Config::dbname`
    pub dbname: Option<String>,
    /// See `tokio_postgres::Config::options`
    pub options: Option<String>,
    /// See `tokio_postgres::Config::application_name`
    pub application_name: Option<String>,
    /// See `tokio_postgres::Config::ssl_mode`
    pub ssl_mode: Option<SslMode>,
    /// This is similar to `hosts` but only allows one host to be specified.
    /// Unlike `tokio-postgres::Config` this structure differenciates between
    /// one host and more than one host. This makes it possible to store this
    /// configuration in an envorinment variable.
    /// See `tokio_postgres::Config::host`
    pub host: Option<String>,
    /// See `tokio_postgres::Config::hosts`
    pub hosts: Option<Vec<String>>,
    /// This is similar to `ports` but only allows one port to be specified.
    /// Unlike `tokio-postgres::Config` this structure differenciates between
    /// one port and more than one port. This makes it possible to store this
    /// configuration in an environment variable.
    /// See `tokio_postgres::Config::port`
    pub port: Option<u16>,
    /// See `tokio_postgres::Config::port`
    pub ports: Option<Vec<u16>>,
    /// See `tokio_postgres::Config::connect_timeout`
    pub connect_timeout: Option<Duration>,
    /// See `tokio_postgres::Config::keepalives`
    pub keepalives: Option<bool>,
    /// See `tokio_postgres::Config::keepalives_idle`
    pub keepalives_idle: Option<Duration>,
    /// See `tokio_postgres::Config::target_session_attrs`
    pub target_session_attrs: Option<TargetSessionAttrs>,
    /// See `tokio_postgres::Config::channel_binding`
    pub channel_binding: Option<ChannelBinding>,
    /// Manager configuration
    pub manager: Option<ManagerConfig>,
    /// Pool configuration
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Create new config instance with default values. This function is
    /// identical to `Config::default`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Create pool using the current configuration
    pub fn create_pool<T>(&self, tls: T) -> Result<Pool, ConfigError>
    where
        T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
        T::Stream: Sync + Send,
        T::TlsConnect: Sync + Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        let pg_config = self.get_pg_config()?;
        let manager_config = self.get_manager_config();
        let manager = crate::Manager::from_config(pg_config, tls, manager_config);
        let pool_config = self.get_pool_config();
        Ok(Pool::from_config(manager, pool_config))
    }
    /// Get `tokio_postgres::Config` which can be used to connect to
    /// the database.
    pub fn get_pg_config(&self) -> Result<tokio_postgres::Config, ConfigError> {
        let mut cfg = tokio_postgres::Config::new();
        if let Some(user) = &self.user {
            cfg.user(user.as_str());
        } else if let Ok(user) = env::var("USER") {
            cfg.user(user.as_str());
        }
        if let Some(password) = &self.password {
            cfg.password(password);
        }
        match &self.dbname {
            Some(dbname) => match dbname.as_str() {
                "" => {
                    return Err(ConfigError::Message(
                        "configuration property \"dbname\" not found".to_string(),
                    ))
                }
                dbname => cfg.dbname(dbname),
            },
            None => {
                return Err(ConfigError::Message(
                    "configuration property \"dbname\" contains an empty string".to_string(),
                ))
            }
        };
        if let Some(options) = &self.options {
            cfg.options(options.as_str());
        }
        if let Some(application_name) = &self.application_name {
            cfg.application_name(application_name.as_str());
        }
        if let Some(host) = &self.host {
            cfg.host(host.as_str());
        }
        if let Some(hosts) = &self.hosts {
            for host in hosts.iter() {
                cfg.host(host.as_str());
            }
        }
        if self.host.is_none() && self.hosts.is_none() {
            // Systems that support it default to unix domain sockets
            #[cfg(unix)]
            {
                cfg.host_path("/run/postgresql");
                cfg.host_path("/var/run/postgresql");
                cfg.host_path("/tmp");
            }
            // Windows and other systems use 127.0.0.1 instead
            #[cfg(not(unix))]
            cfg.host("127.0.0.1");
        }
        if let Some(port) = &self.port {
            cfg.port(*port);
        }
        if let Some(ports) = &self.ports {
            for port in ports.iter() {
                cfg.port(*port);
            }
        }
        if let Some(connect_timeout) = &self.connect_timeout {
            cfg.connect_timeout(*connect_timeout);
        }
        if let Some(keepalives) = &self.keepalives {
            cfg.keepalives(*keepalives);
        }
        if let Some(keepalives_idle) = &self.keepalives_idle {
            cfg.keepalives_idle(*keepalives_idle);
        }
        Ok(cfg)
    }
    /// Get `deadpool_postgres::ManagerConfig` which can be used to
    /// construct a `deadpool::managed::Pool` instance.
    pub fn get_manager_config(&self) -> ManagerConfig {
        self.manager.clone().unwrap_or_default()
    }
    /// Get `deadpool::PoolConfig` which can be used to construct a
    /// `deadpool::managed::Pool` instance.
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.clone().unwrap_or_default()
    }
}
