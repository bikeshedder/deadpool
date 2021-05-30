//! # Deadpool for PostgreSQL [![Latest Version](https://img.shields.io/crates/v/deadpool-postgres.svg)](https://crates.io/crates/deadpool-postgres)
//!
//! Deadpool is a dead simple async pool for connections and objects
//! of any type.
//!
//! This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
//! manager for [`tokio-postgres`](https://crates.io/crates/tokio-postgres)
//! and also provides a `statement` cache by wrapping `tokio_postgres::Client`
//! and `tokio_postgres::Transaction`.
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
//! use deadpool_sqlite::Config;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut cfg = Config::new("db.sqlite3");
//!     let pool = cfg.create_pool();
//!     for i in 1..10 {
//!         let mut conn = pool.get().await.unwrap();
//!         let value: i32 = conn.interact(move |conn| {
//!             let mut stmt = conn.prepare_cached("SELECT 1 + $1").unwrap();
//!             stmt.query_row([&i], |row| row.get(0)).unwrap()
//!         }).await;
//!         assert_eq!(value, i + 1);
//!     }
//! }
//! ```
#![warn(missing_docs, unreachable_pub)]

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;

use deadpool::managed::RecycleError;
use rusqlite::Error;

mod config;
pub use config::Config;

/// Re-export deadpool::managed::PoolConfig
pub use deadpool::managed::PoolConfig;
/// Re-export deadpool::Runtime;
pub use deadpool::Runtime;

/// A type alias for using `deadpool::Pool` with `rusqlite`
pub type Pool = deadpool::managed::Pool<Manager>;

/// A type alias for using `deadpool::PoolError` with `rusqlite`
pub type PoolError = deadpool::managed::PoolError<Error>;

/// A type alias for using `deadpool::Object` with `rusqlite`
pub type Connection = deadpool::managed::Object<Manager>;

/// The manager for creating and recyling SQLite connections
pub struct Manager {
    config: Config,
    recycle_count: AtomicUsize,
}

impl Manager {
    /// Create manager using a `deadpool_sqlite::Config`
    pub fn from_config(config: &Config) -> Self {
        Self {
            config: config.clone(),
            recycle_count: AtomicUsize::new(0),
        }
    }
}

#[async_trait::async_trait]
impl deadpool::managed::Manager for Manager {
    type Type = ConnectionWrapper;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        ConnectionWrapper::open(&self.config).await
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        let recycle_count = self.recycle_count.fetch_add(1, Ordering::Relaxed);
        let n: usize = conn
            .interact(move |conn| conn.query_row("SELECT $1", [recycle_count], |row| row.get(0)))
            .await
            .map_err(|e| RecycleError::Backend(e))?;
        if n == recycle_count {
            Ok(())
        } else {
            Err(RecycleError::Message(String::from(
                "Recycle count mismatch",
            )))
        }
    }
}

/// A wrapper for `rusqlite::Connection` which provides indirect
/// access to it via the `interact` function.
pub struct ConnectionWrapper {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl ConnectionWrapper {
    async fn open(config: &Config) -> Result<Self, Error> {
        let config = config.clone();
        let conn = tokio::task::spawn_blocking(move || rusqlite::Connection::open(config.path))
            .await
            .unwrap()?;
        Ok(ConnectionWrapper {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
    /// Interact with the underlying SQLite connection. This function
    /// expects a closure that takes the connection as parameter. The
    /// closure is executed in a separate thread so that the async runtime
    /// is not blocked.
    pub async fn interact<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rusqlite::Connection) -> R + Send + 'static,
        R: Send + 'static,
    {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || f(&*conn.lock().unwrap()))
            .await
            .unwrap()
    }
}
