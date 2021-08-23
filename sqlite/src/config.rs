use std::path::PathBuf;

use deadpool::{
    managed::{BuildError, PoolConfig},
    Runtime,
};

use crate::{Manager, Pool};

/// Configuration object.
///
/// # Example (from environment)
///
/// By enabling the `serde` feature you can read the configuration using the
/// [`config`](https://crates.io/crates/config) crate as following:
/// ```env
/// SQLITE__PATH=db.sqlite3
/// SQLITE__POOL__MAX_SIZE=16
/// SQLITE__POOL__TIMEOUTS__WAIT__SECS=5
/// SQLITE__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ```rust
/// # use serde_1 as serde;
/// #
/// #[derive(serde::Deserialize)]
/// # #[serde(crate = "serde_1")]
/// struct Config {
///     sqlite: deadpool_sqlite::Config,
/// }
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
    /// Path to SQLite database file.
    pub path: PathBuf,

    /// [`Pool`] configuration.
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Create a new [`Config`] with the given `path` of SQLite database file.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            pool: None,
        }
    }

    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`BuildError`] and [`rusqlite::Error`] for details.
    ///
    /// [`RedisError`]: redis::RedisError
    pub fn create_pool(&self, runtime: Runtime) -> Result<Pool, BuildError<rusqlite::Error>> {
        let manager = Manager::from_config(self, runtime);
        Pool::builder(manager)
            .config(self.get_pool_config())
            .runtime(runtime)
            .build()
    }

    /// Returns [`deadpool::managed::PoolConfig`] which can be used to construct
    /// a [`deadpool::managed::Pool`] instance.
    #[must_use]
    pub fn get_pool_config(&self) -> PoolConfig {
        self.pool.unwrap_or_default()
    }
}
