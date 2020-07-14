//! # Deadpool for Redis [![Latest Version](https://img.shields.io/crates/v/deadpool-redis.svg)](https://crates.io/crates/deadpool-redis)
//!
//! Deadpool is a dead simple async pool for connections and objects
//! of any type.
//!
//! This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
//! manager for [`redis`](https://crates.io/crates/redis).
//!
//! ## Features
//!
//! | Feature | Description | Extra dependencies | Default |
//! | ------- | ----------- | ------------------ | ------- |
//! | `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |
//!
//! ## Example
//!
//! ```rust
//! use deadpool_redis::{cmd, Config};
//! use deadpool_redis::redis::FromRedisValue;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut cfg = Config::default();
//!     cfg.url = Some("redis://127.0.0.1/".to_string());
//!     let pool = cfg.create_pool().unwrap();
//!     {
//!         let mut conn = pool.get().await.unwrap();
//!         cmd("SET")
//!             .arg(&["deadpool/test_key", "42"])
//!             .execute_async(&mut conn)
//!             .await.unwrap();
//!     }
//!     {
//!         let mut conn = pool.get().await.unwrap();
//!         let value: String = cmd("GET")
//!             .arg(&["deadpool/test_key"])
//!             .query_async(&mut conn)
//!             .await.unwrap();
//!         assert_eq!(value, "42".to_string());
//!     }
//! }
//! ```
//!
//! ## Example with `config` and `dotenv` crate
//!
//! ```rust
//! use deadpool_redis::cmd;
//! use deadpool_redis::redis::FromRedisValue;
//! use dotenv::dotenv;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct Config {
//!     #[serde(default)]
//!     redis: deadpool_redis::Config
//! }
//!
//! impl Config {
//!     pub fn from_env() -> Result<Self, ::config_crate::ConfigError> {
//!         let mut cfg = ::config_crate::Config::new();
//!         cfg.merge(::config_crate::Environment::new().separator("__"))?;
//!         cfg.try_into()
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     dotenv().ok();
//!     let cfg = Config::from_env().unwrap();
//!     let pool = cfg.redis.create_pool().unwrap();
//!     {
//!         let mut conn = pool.get().await.unwrap();
//!         cmd("SET")
//!             .arg(&["deadpool/test_key", "42"])
//!             .execute_async(&mut conn)
//!             .await.unwrap();
//!     }
//!     {
//!         let mut conn = pool.get().await.unwrap();
//!         let value: String = cmd("GET")
//!             .arg(&["deadpool/test_key"])
//!             .query_async(&mut conn)
//!             .await.unwrap();
//!         assert_eq!(value, "42".to_string());
//!     }
//! }
//! ```
//!
//! ## License
//!
//! Licensed under either of
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
//! - MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
#![warn(missing_docs)]

use std::ops::{Deref, DerefMut};

use async_trait::async_trait;
use redis::{
    aio::Connection as RedisConnection, Client, IntoConnectionInfo, RedisError, RedisResult,
};

/// A type alias for using `deadpool::Pool` with `redis`
pub type Pool = deadpool::managed::Pool<ConnectionWrapper, RedisError>;

/// A type alias for using `deadpool::PoolError` with `redis`
pub type PoolError = deadpool::managed::PoolError<RedisError>;

/// A type alias for using `deadpool::Object` with `redis`
pub type Connection = deadpool::managed::Object<ConnectionWrapper, RedisError>;

type RecycleResult = deadpool::managed::RecycleResult<RedisError>;

/// Re-export redis crate
pub use redis;

mod config;
pub use config::Config;
mod cmd_wrapper;
pub use cmd_wrapper::{cmd, Cmd};
mod pipeline_wrapper;
pub use pipeline_wrapper::{pipe, Pipeline};

/// A wrapper for `redis::Connection`. The `query_async` and `execute_async`
/// functions of `redis::Cmd` and `redis::Pipeline` consume the connection.
/// This wrapper makes it possible to replace the internal connection after
/// executing a query.
pub struct ConnectionWrapper {
    conn: RedisConnection,
}

impl Deref for ConnectionWrapper {
    type Target = RedisConnection;
    fn deref(&self) -> &RedisConnection {
        &self.conn
    }
}

impl DerefMut for ConnectionWrapper {
    fn deref_mut(&mut self) -> &mut RedisConnection {
        &mut self.conn
    }
}

/// The manager for creating and recyling lapin connections
pub struct Manager {
    client: Client,
}

impl Manager {
    /// Create manager using `PgConfig` and a `TlsConnector`
    pub fn new<T: IntoConnectionInfo>(params: T) -> RedisResult<Self> {
        Ok(Self {
            client: Client::open(params)?,
        })
    }
}

#[async_trait]
impl deadpool::managed::Manager<ConnectionWrapper, RedisError> for Manager {
    async fn create(&self) -> Result<ConnectionWrapper, RedisError> {
        let conn = self.client.get_async_connection().await?;
        Ok(ConnectionWrapper { conn })
    }

    async fn recycle(&self, conn: &mut ConnectionWrapper) -> RecycleResult {
        match cmd("PING").execute_async(conn).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
