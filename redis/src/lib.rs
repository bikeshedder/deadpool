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
//! | `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1`, `redis/tokio-comp` | yes |
//! | `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate | `deadpool/rt_async-std_1`, `redis/async-std-comp` | no |
//!
//! ## Example
//!
//! ```rust,ignore
//! use deadpool_redis::redis::{cmd, Config, FromRedisValue};
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
//!             .query_async::<_, ()>(&mut conn)
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
//! use deadpool_redis::redis::{cmd, FromRedisValue};
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
//!             .query_async::<_, ()>(&mut conn)
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
//! ## FAQ
//!
//! - **How can I enable features of the `redis` crate?**
//!
//!   Make sure that you depend on the same version of `redis` as
//!   `deadpool-redis` does and enable the needed features in your own
//!   `Crate.toml` file:
//!
//!   ```toml
//!   [dependencies]
//!   deadpool-redis = { version = "0.8", features = ["config"] }
//!   redis = { version = "0.20", default-features = false, features = ["tls"] }
//!   ```
//!
//! ## License
//!
//! Licensed under either of
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
//! - MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
#![warn(missing_docs, unreachable_pub)]

use std::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
/// Re-export deadpool::managed::PoolConfig
pub use deadpool::managed::PoolConfig;
use deadpool::managed::RecycleError;
/// Re-export deadpool::Runtime;
pub use deadpool::Runtime;
use redis::{
    aio::{Connection as RedisConnection, ConnectionLike},
    Client, IntoConnectionInfo, RedisError, RedisResult,
};

/// A type alias for using `deadpool::Pool` with `redis`
pub type Pool = deadpool::managed::Pool<Manager, ConnectionWrapper>;

/// A type alias for using `deadpool::PoolError` with `redis`
pub type PoolError = deadpool::managed::PoolError<RedisError>;

/// A type alias for using `deadpool::Object` with `redis`
pub type Connection = deadpool::managed::Object<Manager>;

type RecycleResult = deadpool::managed::RecycleResult<RedisError>;

/// Re-export redis crate
pub use redis;

mod config;
pub use config::Config;

/// A wrapper for `redis::Connection`. The `query_async` and `execute_async`
/// functions of `redis::Cmd` and `redis::Pipeline` consume the connection.
/// This wrapper makes it possible to replace the internal connection after
/// executing a query.
pub struct ConnectionWrapper {
    conn: Connection,
}

impl ConnectionWrapper {
    /// Take this object from the pool permanently. This reduces the size of
    /// the pool.
    pub fn take(this: Self) -> RedisConnection {
        Connection::take(this.conn)
    }
}

impl From<Connection> for ConnectionWrapper {
    fn from(conn: Connection) -> Self {
        Self { conn }
    }
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

impl AsRef<redis::aio::Connection> for ConnectionWrapper {
    fn as_ref(&self) -> &redis::aio::Connection {
        &self.conn
    }
}

impl AsMut<redis::aio::Connection> for ConnectionWrapper {
    fn as_mut(&mut self) -> &mut redis::aio::Connection {
        &mut self.conn
    }
}

impl ConnectionLike for ConnectionWrapper {
    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        self.conn.req_packed_command(cmd)
    }
    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        self.conn.req_packed_commands(cmd, offset, count)
    }
    fn get_db(&self) -> i64 {
        self.conn.get_db()
    }
}

/// The manager for creating and recyling lapin connections
pub struct Manager {
    client: Client,
    ping_number: AtomicUsize,
}

impl Manager {
    /// Create manager using `PgConfig` and a `TlsConnector`
    pub fn new<T: IntoConnectionInfo>(params: T) -> RedisResult<Self> {
        Ok(Self {
            client: Client::open(params)?,
            ping_number: AtomicUsize::new(0),
        })
    }
}

#[async_trait]
impl deadpool::managed::Manager for Manager {
    type Type = RedisConnection;
    type Error = RedisError;
    async fn create(&self) -> Result<RedisConnection, RedisError> {
        let conn = self.client.get_async_connection().await?;
        Ok(conn)
    }
    async fn recycle(&self, conn: &mut RedisConnection) -> RecycleResult {
        let ping_number = self.ping_number.fetch_add(1, Ordering::Relaxed).to_string();
        match redis::cmd(&format!("PING {}", ping_number))
            .query_async::<_, String>(conn)
            .await
        {
            Ok(n) => {
                if n == ping_number {
                    Ok(())
                } else {
                    Err(RecycleError::Message(String::from("Invalid PING response")))
                }
            }
            Err(e) => Err(e.into()),
        }
    }
}
