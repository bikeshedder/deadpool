#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

mod config;

use std::{
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

use deadpool::{async_trait, managed};
use redis::{aio::ConnectionLike, IntoConnectionInfo, RedisError, RedisResult};

pub use redis;
pub use redis_cluster_async::{Client, Connection as RedisConnection};

pub use self::config::{Config, ConfigError};

pub use deadpool::managed::reexports::*;
deadpool::managed_reexports!(
    "redis_cluster",
    Manager,
    Connection,
    RedisError,
    ConfigError
);

type RecycleResult = managed::RecycleResult<RedisError>;

/// Wrapper around [`redis_cluster_async::Connection`].
///
/// This structure implements [`redis::aio::ConnectionLike`] and can therefore
/// be used just like a regular [`redis_cluster_async::Connection`].
#[allow(missing_debug_implementations)] // `redis_cluster_async::Connection: !Debug`
pub struct Connection {
    conn: Object,
}

impl Connection {
    /// Takes this [`Connection`] from its [`Pool`] permanently.
    ///
    /// This reduces the size of the [`Pool`].
    #[must_use]
    pub fn take(this: Self) -> RedisConnection {
        Object::take(this.conn)
    }
}

impl From<Object> for Connection {
    fn from(conn: Object) -> Self {
        Self { conn }
    }
}

impl Deref for Connection {
    type Target = RedisConnection;

    fn deref(&self) -> &RedisConnection {
        &self.conn
    }
}

impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut RedisConnection {
        &mut self.conn
    }
}

impl AsRef<RedisConnection> for Connection {
    fn as_ref(&self) -> &RedisConnection {
        &self.conn
    }
}

impl AsMut<RedisConnection> for Connection {
    fn as_mut(&mut self) -> &mut RedisConnection {
        &mut self.conn
    }
}

impl ConnectionLike for Connection {
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

/// [`Manager`] for creating and recycling [`redis_cluster_async`] connections.
///
/// [`Manager`]: managed::Manager
pub struct Manager {
    client: Client,
    ping_number: AtomicUsize,
}

// `redis_cluster_async::Client: !Debug`
impl std::fmt::Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manager")
            .field("client", &format!("{:p}", &self.client))
            .field("ping_number", &self.ping_number)
            .finish()
    }
}

impl Manager {
    /// Creates a new [`Manager`] from the given `params`.
    ///
    /// # Errors
    ///
    /// If establishing a new [`Client`] fails.
    pub fn new<T: IntoConnectionInfo>(params: Vec<T>) -> RedisResult<Self> {
        Ok(Self {
            client: Client::open(params)?,
            ping_number: AtomicUsize::new(0),
        })
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = RedisConnection;
    type Error = RedisError;

    async fn create(&self) -> Result<RedisConnection, RedisError> {
        let conn = self.client.get_connection().await?;
        Ok(conn)
    }

    async fn recycle(&self, conn: &mut RedisConnection) -> RecycleResult {
        let ping_number = self.ping_number.fetch_add(1, Ordering::Relaxed).to_string();
        let n = redis::cmd("PING")
            .arg(&ping_number)
            .query_async::<_, String>(conn)
            .await?;
        if n == ping_number {
            Ok(())
        } else {
            Err(managed::RecycleError::StaticMessage(
                "Invalid PING response",
            ))
        }
    }
}
