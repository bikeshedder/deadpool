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
//! use deadpool_redis::{Manager, Pool};
//! use redis::FromRedisValue;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mgr = Manager::new("redis://127.0.0.1/").unwrap();
//!     let pool = Pool::new(mgr, 16);
//!     {
//!         let mut conn = pool.get().await.unwrap();
//!         let mut cmd = redis::cmd("SET");
//!         cmd.arg(&["deadpool/test_key", "42"]);
//!         conn.query(&cmd).await.unwrap();
//!     }
//!     {
//!         let mut conn = pool.get().await.unwrap();
//!         let mut cmd = redis::cmd("GET");
//!         cmd.arg(&["deadpool/test_key"]);
//!         let value = conn.query(&cmd).await.unwrap();
//!         assert_eq!(String::from_redis_value(&value).unwrap(), "42".to_string());
//!     }
//! }
//! ```
#![warn(missing_docs)]

use async_trait::async_trait;
use futures::compat::Future01CompatExt;
use redis::{
    aio::Connection as RedisConnection,
    Client,
    IntoConnectionInfo,
    RedisError,
    RedisResult,
};

/// A type alias for using `deadpool::Pool` with `redis`
pub type Pool = deadpool::Pool<Connection, RedisError>;

/// A type alias for using `deadpool::Object` with `redis`
pub struct Connection {
    conn: Option<RedisConnection>
}

impl Connection {
    /// Execute query
    pub async fn query(&mut self, cmd: &redis::Cmd) -> RedisResult<redis::Value>
    {
        if let Some(conn) = self.conn.take() {
            let (conn, result) = cmd.query_async(conn).compat().await?;
            self.conn.replace(conn);
            Ok(result)
        } else {
            Err(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "deadpool.redis: Connection to server lost due to previous query"
            )))
        }
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
            client: Client::open(params)?
        })
    }
}

#[async_trait]
impl deadpool::Manager<Connection, RedisError> for Manager
{
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
                "deadpool.redis: Connection could not be recycled"
            )))
        }
    }
}
