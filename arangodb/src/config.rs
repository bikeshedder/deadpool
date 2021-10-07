use deadpool::{managed::BuildError, Runtime};
#[cfg(feature = "serde")]
use serde_1::Deserialize;
use url::Url;

use crate::{Pool, PoolConfig};

/// Configuration object.
///
/// # Example (from environment)
///
/// By enabling the `serde` feature you can read the configuration using the
/// [`config`](https://crates.io/crates/config) crate as following:
/// ```env
/// ARANGODB__URL=arangodb.example.com
/// ARANGODB__USERNAME=root
/// ARANGODB__PASSWORD=deadpool
/// ARANGODB__USE_JWT=true
/// ARANGODB__POOL__MAX_SIZE=16
/// ARANGODB__POOL__TIMEOUTS__WAIT__SECS=2
/// ARANGODB__POOL__TIMEOUTS__WAIT__NANOS=0
/// ```
/// ```rust
/// # use serde_1 as serde;
/// #
/// #[derive(serde::Deserialize)]
/// # #[serde(crate = "serde_1")]
/// struct Config {
///     arango: deadpool_arangodb::Config,
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
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde_1::Deserialize))]
#[cfg_attr(feature = "serde", serde(crate = "serde_1"))]
pub struct Config {
    /// ArangoDB URL.
    ///
    /// See [Arangors Connection](arangors/connection/struct.GenericConnection.html#method.establish_jwt).
    pub url: Option<String>,
    /// ArangoDB username.
    /// If you have not manually created a new user on a ArangoDB instance, then this must be `root`.
    ///
    /// See [Arangors Connection](arangors/connection/struct.GenericConnection.html#method.establish_jwt).
    pub username: Option<String>,
    /// ArangoDB password.
    ///
    /// See [Arangors Connection](arangors/connection/struct.GenericConnection.html#method.establish_jwt).
    pub password: Option<String>,
    /// If jwt authentication should be used. JWT token expires after 1 month.
    ///
    /// See [Arangors Connection](arangors/connection/struct.GenericConnection.html#method.establish_jwt).
    pub use_jwt: bool,

    /// [`Pool`] configuration.
    pub pool: Option<PoolConfig>,
}

impl Config {
    /// Creates a new [`Config`] from the given URL.
    ///
    /// Url format is: `http://username:password@localhost:8529/?use_jwt=true`. If `use_jwt` is missing, then it defaults to `true`.
    pub fn from_url<U: Into<String>>(url: U) -> Result<Self, BuildError<()>> {
        let url = url.into();
        let url = Url::parse(&url)
            .map_err(|e| BuildError::Config(format!("Could not extract a valid config from url: `{}` - Error: {}", url, e)))?;

        let use_jwt = url.query_pairs()
            .filter(|(name, _)| name == "use_jwt")
            .map(|(_, value)| value.to_string())
            .next();
        let use_jwt = match use_jwt {
            Some(use_jwt) => use_jwt.parse()
                .map_err(|e| BuildError::Config(format!("Could not parse `use_jwt` value: `{}` - Error: {}", use_jwt, e)))?,
            None => true,
        };

        Ok(Config {
            url: Some(format!("{}://{}:{}", url.scheme(), url.host_str().unwrap(), url.port_or_known_default().unwrap())),
            username: Some(url.username().to_string()),
            password: url.password().map(ToString::to_string),
            use_jwt,
            pool: None,
        })
    }


    /// Creates a new [`Pool`] using this [`Config`].
    ///
    /// # Errors
    ///
    /// See [`BuildError`] and [`ClientError`] for details.
    ///
    /// [`ClientError`]: arangors::ClientError
    pub fn create_pool(&self, runtime: Runtime) -> Result<Pool, BuildError<arangors::ClientError>> {
        let manager = match (&self.url, &self.username, &self.password) {
            (Some(_), Some(_), Some(_)) => crate::Manager::from_config(self.clone())?,
            _ => {
                return Err(BuildError::Config("url, username and password must be specified.".into()))
            }
        };

        let pool_config = self.get_pool_config();
        Pool::builder(manager)
            .config(pool_config)
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

impl Default for Config {
    fn default() -> Self {
        Self {
            url: None,
            username: None,
            password: None,
            use_jwt: true,
            pool: None,
        }
    }
}
