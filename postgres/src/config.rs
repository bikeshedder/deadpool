//! Configuration used for [`Pool`] creation.

use std::{env, fmt, time::Duration};

#[cfg(feature = "serde")]
use serde_1 as serde;
use tokio_postgres::{
    config::{
        ChannelBinding as PgChannelBinding, SslMode as PgSslMode,
        TargetSessionAttrs as PgTargetSessionAttrs,
    },
    tls::{MakeTlsConnect, TlsConnect},
    Socket,
};

use crate::{CreatePoolError, PoolBuilder, Runtime};

use super::{Pool, PoolConfig};

/// Configuration object.
///
/// # Example (from environment)
///
/// By enabling the `serde` feature you can read the configuration using the
/// [`config`](https://crates.io/crates/config) crate as following:
/// ```env
/// PG__HOST=pg.example.com
/// PG__USER=john_doe
/// PG__PASSWORD=topsecret
/// PG__DBNAME=example
/// PG__POOL__MAX_SIZE=16
/// PG__POOL__TIMEOUTS__WAIT__SECS=5
/// PG__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ```rust
/// # use serde_1 as serde;
/// #
/// #[derive(serde::Deserialize, serde::Serialize)]
/// # #[serde(crate = "serde_1")]
/// struct Config {
///     pg: deadpool_postgres::Config,
/// }
/// impl Config {
///     pub fn from_env() -> Result<Self, config::ConfigError> {
///         let mut cfg = config::Config::builder()
///            .add_source(config::Environment::default().separator("__"))
///            .build()?;
///            cfg.try_deserialize()
///     }
/// }
/// ```
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde_1"))]
pub struct Config {
    /// See [`tokio_postgres::Config::user`].
    pub user: Option<String>,
    /// See [`tokio_postgres::Config::password`].
    pub password: Option<String>,
    /// See [`tokio_postgres::Config::dbname`].
    pub dbname: Option<String>,
    /// See [`tokio_postgres::Config::options`].
    pub options: Option<String>,
    /// See [`tokio_postgres::Config::application_name`].
    pub application_name: Option<String>,
    /// See [`tokio_postgres::Config::ssl_mode`].
    pub ssl_mode: Option<SslMode>,
    /// This is similar to [`Config::hosts`] but only allows one host to be
    /// specified.
    ///
    /// Unlike [`tokio_postgres::Config`] this structure differentiates between
    /// one host and more than one host. This makes it possible to store this
    /// configuration in an environment variable.
    ///
    /// See [`tokio_postgres::Config::host`].
    pub host: Option<String>,
    /// See [`tokio_postgres::Config::host`].
    pub hosts: Option<Vec<String>>,
    /// This is similar to [`Config::ports`] but only allows one port to be
    /// specified.
    ///
    /// Unlike [`tokio_postgres::Config`] this structure differentiates between
    /// one port and more than one port. This makes it possible to store this
    /// configuration in an environment variable.
    ///
    /// See [`tokio_postgres::Config::port`].
    pub port: Option<u16>,
    /// See [`tokio_postgres::Config::port`].
    pub ports: Option<Vec<u16>>,
    /// See [`tokio_postgres::Config::connect_timeout`].
    pub connect_timeout: Option<Duration>,
    /// See [`tokio_postgres::Config::keepalives`].
    pub keepalives: Option<bool>,
    /// See [`tokio_postgres::Config::keepalives_idle`].
    pub keepalives_idle: Option<Duration>,
    /// See [`tokio_postgres::Config::target_session_attrs`].
    pub target_session_attrs: Option<TargetSessionAttrs>,
    /// See [`tokio_postgres::Config::channel_binding`].
    pub channel_binding: Option<ChannelBinding>,

    /// [`Manager`] configuration.
    ///
    /// [`Manager`]: super::Manager
    pub manager: Option<ManagerConfig>,

    /// [`Pool`] configuration.
    pub pool: Option<PoolConfig>,
}

/// This error is returned if there is something wrong with the configuration
#[derive(Copy, Clone, Debug)]
pub enum ConfigError {
    /// This variant is returned if the `dbname` is missing from the config
    DbnameMissing,
    /// This variant is returned if the `dbname` contains an empty string
    DbnameEmpty,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DbnameMissing => write!(f, "configuration property \"dbname\" not found"),
            Self::DbnameEmpty => write!(
                f,
                "configuration property \"dbname\" contains an empty string",
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Create a new [`Config`] instance with default values. This function is
    /// identical to [`Config::default()`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`CreatePoolError`] for details.
    pub fn create_pool<T>(&self, runtime: Option<Runtime>, tls: T) -> Result<Pool, CreatePoolError>
    where
        T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
        T::Stream: Sync + Send,
        T::TlsConnect: Sync + Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        let mut builder = self.builder(tls).map_err(CreatePoolError::Config)?;
        if let Some(runtime) = runtime {
            builder = builder.runtime(runtime);
        }
        builder.build().map_err(CreatePoolError::Build)
    }

    /// Creates a new [`PoolBuilder`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`ConfigError`] and [`tokio_postgres::Error`] for details.
    pub fn builder<T>(&self, tls: T) -> Result<PoolBuilder, ConfigError>
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
        Ok(Pool::builder(manager).config(pool_config))
    }

    /// Returns [`tokio_postgres::Config`] which can be used to connect to
    /// the database.
    #[allow(unused_results)]
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
                "" => return Err(ConfigError::DbnameMissing),
                dbname => cfg.dbname(dbname),
            },
            None => return Err(ConfigError::DbnameEmpty),
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
            // Systems that support it default to unix domain sockets.
            #[cfg(unix)]
            {
                cfg.host_path("/run/postgresql");
                cfg.host_path("/var/run/postgresql");
                cfg.host_path("/tmp");
            }
            // Windows and other systems use 127.0.0.1 instead.
            #[cfg(not(unix))]
            cfg.host("127.0.0.1");
        }
        if let Some(port) = self.port {
            cfg.port(port);
        }
        if let Some(ports) = &self.ports {
            for port in ports.iter() {
                cfg.port(*port);
            }
        }
        if let Some(connect_timeout) = self.connect_timeout {
            cfg.connect_timeout(connect_timeout);
        }
        if let Some(keepalives) = self.keepalives {
            cfg.keepalives(keepalives);
        }
        if let Some(keepalives_idle) = self.keepalives_idle {
            cfg.keepalives_idle(keepalives_idle);
        }
        if let Some(mode) = self.ssl_mode {
            cfg.ssl_mode(mode.into());
        }
        Ok(cfg)
    }

    /// Returns [`ManagerConfig`] which can be used to construct a
    /// [`deadpool::managed::Pool`] instance.
    #[must_use]
    pub fn get_manager_config(&self) -> ManagerConfig {
        self.manager.clone().unwrap_or_default()
    }

    /// Returns [`deadpool::managed::PoolConfig`] which can be used to construct
    /// a [`deadpool::managed::Pool`] instance.
    #[must_use]
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.unwrap_or_default()
    }
}

/// Possible methods of how a connection is recycled.
///
/// The default is [`Fast`] which does not check the connection health or
/// perform any clean-up queries.
///
/// [`Fast`]: RecyclingMethod::Fast
/// [`Verified`]: RecyclingMethod::Verified
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde_1"))]
pub enum RecyclingMethod {
    /// Only run [`Client::is_closed()`][1] when recycling existing connections.
    ///
    /// Unless you have special needs this is a safe choice.
    ///
    /// [1]: tokio_postgres::Client::is_closed
    Fast,

    /// Run [`Client::is_closed()`][1] and execute a test query.
    ///
    /// This is slower, but guarantees that the database connection is ready to
    /// be used. Normally, [`Client::is_closed()`][1] should be enough to filter
    /// out bad connections, but under some circumstances (i.e. hard-closed
    /// network connections) it's possible that [`Client::is_closed()`][1]
    /// returns `false` while the connection is dead. You will receive an error
    /// on your first query then.
    ///
    /// [1]: tokio_postgres::Client::is_closed
    Verified,

    /// Like [`Verified`] query method, but instead use the following sequence
    /// of statements which guarantees a pristine connection:
    /// ```sql
    /// CLOSE ALL;
    /// SET SESSION AUTHORIZATION DEFAULT;
    /// RESET ALL;
    /// UNLISTEN *;
    /// SELECT pg_advisory_unlock_all();
    /// DISCARD TEMP;
    /// DISCARD SEQUENCES;
    /// ```
    ///
    /// This is similar to calling `DISCARD ALL`. but doesn't call
    /// `DEALLOCATE ALL` and `DISCARD PLAN`, so that the statement cache is not
    /// rendered ineffective.
    ///
    /// [`Verified`]: RecyclingMethod::Verified
    Clean,

    /// Like [`Verified`] but allows to specify a custom SQL to be executed.
    ///
    /// [`Verified`]: RecyclingMethod::Verified
    Custom(String),
}

impl Default for RecyclingMethod {
    fn default() -> Self {
        Self::Fast
    }
}

impl RecyclingMethod {
    const DISCARD_SQL: &'static str = "\
        CLOSE ALL; \
        SET SESSION AUTHORIZATION DEFAULT; \
        RESET ALL; \
        UNLISTEN *; \
        SELECT pg_advisory_unlock_all(); \
        DISCARD TEMP; \
        DISCARD SEQUENCES;\
    ";

    /// Returns SQL query to be executed when recycling a connection.
    pub fn query(&self) -> Option<&str> {
        match self {
            Self::Fast => None,
            Self::Verified => Some(""),
            Self::Clean => Some(Self::DISCARD_SQL),
            Self::Custom(sql) => Some(sql),
        }
    }
}

/// Configuration object for a [`Manager`].
///
/// This currently only makes it possible to specify which [`RecyclingMethod`]
/// should be used when retrieving existing objects from the [`Pool`].
///
/// [`Manager`]: super::Manager
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde_1"))]
pub struct ManagerConfig {
    /// Method of how a connection is recycled. See [`RecyclingMethod`].
    pub recycling_method: RecyclingMethod,
}

/// Properties required of a session.
///
/// This is a 1:1 copy of the [`PgTargetSessionAttrs`] enumeration.
/// This is duplicated here in order to add support for the
/// [`serde::Deserialize`] trait which is required for the [`serde`] support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde_1"))]
#[non_exhaustive]
pub enum TargetSessionAttrs {
    /// No special properties are required.
    Any,

    /// The session must allow writes.
    ReadWrite,
}

impl From<TargetSessionAttrs> for PgTargetSessionAttrs {
    fn from(attrs: TargetSessionAttrs) -> Self {
        match attrs {
            TargetSessionAttrs::Any => Self::Any,
            TargetSessionAttrs::ReadWrite => Self::ReadWrite,
        }
    }
}

/// TLS configuration.
///
/// This is a 1:1 copy of the [`PgSslMode`] enumeration.
/// This is duplicated here in order to add support for the
/// [`serde::Deserialize`] trait which is required for the [`serde`] support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde_1"))]
#[non_exhaustive]
pub enum SslMode {
    /// Do not use TLS.
    Disable,

    /// Attempt to connect with TLS but allow sessions without.
    Prefer,

    /// Require the use of TLS.
    Require,
}

impl From<SslMode> for PgSslMode {
    fn from(mode: SslMode) -> Self {
        match mode {
            SslMode::Disable => Self::Disable,
            SslMode::Prefer => Self::Prefer,
            SslMode::Require => Self::Require,
        }
    }
}

/// Channel binding configuration.
///
/// This is a 1:1 copy of the [`PgChannelBinding`] enumeration.
/// This is duplicated here in order to add support for the
/// [`serde::Deserialize`] trait which is required for the [`serde`] support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde_1"))]
#[non_exhaustive]
pub enum ChannelBinding {
    /// Do not use channel binding.
    Disable,

    /// Attempt to use channel binding but allow sessions without.
    Prefer,

    /// Require the use of channel binding.
    Require,
}

impl From<ChannelBinding> for PgChannelBinding {
    fn from(cb: ChannelBinding) -> Self {
        match cb {
            ChannelBinding::Disable => Self::Disable,
            ChannelBinding::Prefer => Self::Prefer,
            ChannelBinding::Require => Self::Require,
        }
    }
}
