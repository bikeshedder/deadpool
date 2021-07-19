use deadpool::{
    managed::{BuildError, PoolConfig},
    Runtime,
};

use crate::{Manager, Pool};

/// Configuration object. By enabling the `config` feature you can
/// read the configuration using the [`config`](https://crates.io/crates/config)
/// crate.
///
/// ## Example environment
/// ```env
/// SQLITE__PATH=db.sqlite3
/// SQLITE__POOL__MAX_SIZE=16
/// SQLITE__POOL__TIMEOUTS__WAIT__SECS=5
/// SQLITE__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
///
/// ## Example usage
/// ```rust,ignore
/// struct Config {
///     sqlite: deadpool_postgres::Config,
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
    /// Path to database file
    pub path: String,
    /// Pool configuration
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Create new config instance
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_owned(),
            pool: None,
        }
    }
    /// Create pool using the current configuration
    pub fn create_pool(&self, runtime: Runtime) -> Result<Pool, BuildError<rusqlite::Error>> {
        let manager = Manager::from_config(&self, runtime.clone());
        Pool::builder(manager)
            .config(self.get_pool_config())
            .runtime(runtime)
            .build()
    }
    /// Get `deadpool::PoolConfig` which can be used to construct a
    /// `deadpool::managed::Pool` instance.
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.clone().unwrap_or_default()
    }
}
