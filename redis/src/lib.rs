//! Deadpool simple async pool for Redis connections.
//!
//! This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
//! manager for [`redis`](https://crates.io/crates/redis).
//!
//! You should not need to use `deadpool` directly. Use the `Pool` type
//! provided by this crate instead.
//!
//! # Example
//!
//! ```rust
//! use std::env;
//!
//! use deadpool_redis::{cmd, Manager, Pool};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mgr = Manager::new("redis://127.0.0.1/").unwrap();
//!     let pool = Pool::new(mgr, 16);
//!     {
//!         let mut conn = pool.get().await.unwrap();
//!         cmd("SET")
//!             .arg(&["deadpool/test_key", "42"])
//!             .execute(&mut conn)
//!             .await.unwrap();
//!     }
//!     {
//!         let mut conn = pool.get().await.unwrap();
//!         let value = cmd("GET")
//!             .arg(&["deadpool/test_key"])
//!             .query::<String>(&mut conn)
//!             .await.unwrap();
//!         assert_eq!(value, "42".to_string());
//!     }
//! }
//! ```
#![warn(missing_docs)]

use async_trait::async_trait;
use futures::compat::Future01CompatExt;
use redis::{
    aio::Connection as RedisConnection, Client, IntoConnectionInfo, RedisError,
    RedisResult,
};

/// A type alias for using `deadpool::Pool` with `redis`
pub type Pool = deadpool::Pool<Connection, RedisError>;

mod cmd_wrapper;
pub use cmd_wrapper::{cmd, Cmd};
mod pipeline_wrapper;
pub use pipeline_wrapper::{pipe, Pipeline};

/// A type alias for using `deadpool::Object` with `redis`
pub struct Connection {
    conn: Option<RedisConnection>,
}

impl Connection {

    fn _take_conn(&mut self) -> RedisResult<RedisConnection> {
        if let Some(conn) = self.conn.take() {
            Ok(conn)
        } else {
            Err(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "deadpool.redis: Connection to server lost due to previous query",
            )))
        }
    }

    fn _replace_conn(&mut self, conn: RedisConnection) {
        self.conn = Some(conn)
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
impl deadpool::Manager<Connection, RedisError> for Manager {
    async fn create(&self) -> Result<Connection, RedisError> {
        let conn = self.client.get_async_connection().compat().await?;
        Ok(Connection { conn: Some(conn) })
    }
    async fn recycle(&self, conn: &mut Connection) -> Result<(), RedisError> {
        if conn.conn.is_some() {
            Ok(())
        } else {
            Err(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "deadpool.redis: Connection could not be recycled",
            )))
        }
    }
}
