use std::env;
use std::path::Path;
use std::time::Duration;

#[cfg(feature = "config")]
use ::config_crate::{ConfigError, Environment};
use deadpool::managed::PoolConfig;
use tokio_postgres::config::{
    ChannelBinding as PgChannelBinding, SslMode as PgSslMode,
    TargetSessionAttrs as PgTargetSessionAttrs,
};
use tokio_postgres::tls::{MakeTlsConnect, TlsConnect};
use tokio_postgres::Socket;

use crate::Pool;

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

/// Configuration object. By enabling the `config` feature you can
/// read the configuration using the [`config`](https://crates.io/crates/config)
/// crate.
/// ## Example environment
/// ```env
/// PG_HOST = pg.example.com
/// PG_USER = john_doe
/// PG_PASSWORD = topsecret
/// PG_DBNAME = example
/// PG_POOL.TIMEOUTS.WAIT.SECS = 5
/// PG_POOL.TIMEOUTS.WAIT.NANOS = 0
/// ```
/// ## Example usage
/// ```rust,ignore
/// Config::from_env("PG");
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct Config {
    user: Option<String>,
    password: Option<String>,
    dbname: Option<String>,
    options: Option<String>,
    application_name: Option<String>,
    ssl_mode: Option<SslMode>,
    host: Option<String>,
    hosts: Option<Vec<String>>,
    port: Option<u16>,
    ports: Option<Vec<u16>>,
    connect_timeout: Option<Duration>,
    keepalives: Option<bool>,
    keepalives_idle: Option<Duration>,
    target_session_attrs: Option<TargetSessionAttrs>,
    channel_binding: Option<ChannelBinding>,
    pool: Option<PoolConfig>,
}

impl Config {
    /// Create configuration from environment variables.
    #[cfg(feature = "config")]
    pub fn from_env(prefix: &str) -> Result<Self, ConfigError> {
        let mut cfg = ::config_crate::Config::new();
        cfg.merge(Environment::with_prefix(prefix))?;
        cfg.try_into()
    }
    /// Create pool using the current configuration
    pub fn create_pool<T>(&self, tls: T) -> Pool
    where
        T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
        T::Stream: Sync + Send,
        T::TlsConnect: Sync + Send,
        <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
    {
        let pg_config = self.get_pg_config();
        let manager = crate::Manager::new(pg_config, tls);
        let pool_config = self.get_pool_config();
        Pool::from_config(manager, pool_config)
    }
    /// Get `tokio_postgres::Config` which can be used to connect to
    /// the database.
    pub fn get_pg_config(&self) -> tokio_postgres::Config {
        let mut cfg = tokio_postgres::Config::new();
        if let Some(user) = &self.user {
            cfg.user(user.as_str());
        } else if let Ok(user) = env::var("USER") {
            cfg.user(user.as_str());
        }
        if let Some(password) = &self.password {
            cfg.password(password);
        }
        if let Some(dbname) = &self.dbname {
            cfg.dbname(dbname.as_str());
        }
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
        } else if Path::new("/run/postgresql").exists() {
            cfg.host_path("/run/postgresql");
        } else if Path::new("/var/run/postgresql").exists() {
            cfg.host_path("/var/run/postgresql");
        } else {
            cfg.host_path("/tmp");
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
        cfg
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
            user: None,
            password: None,
            dbname: None,
            options: None,
            application_name: None,
            ssl_mode: None,
            host: None,
            hosts: None,
            port: None,
            ports: None,
            connect_timeout: None,
            keepalives: None,
            keepalives_idle: None,
            target_session_attrs: None,
            channel_binding: None,
            pool: None,
        }
    }
}
