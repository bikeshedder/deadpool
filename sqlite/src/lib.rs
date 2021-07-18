//! # Deadpool for SQLite [![Latest Version](https://img.shields.io/crates/v/deadpool-sqlite.svg)](https://crates.io/crates/deadpool-sqlite)
//!
//! Deadpool is a dead simple async pool for connections and objects
//! of any type.
//!
//! This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
//! manager for [`rusqlite`](https://crates.io/crates/rusqlite)
//! and provides a wrapper that ensures correct use of the connection
//! inside a separate thread.
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
//!     let conn = pool.get().await.unwrap();
//!     let result: i64 = conn
//!         .interact(|conn| {
//!             let mut stmt = conn.prepare("SELECT 1")?;
//!             let mut rows = stmt.query([])?;
//!             let row = rows.next()?.unwrap();
//!             row.get(0)
//!         })
//!         .await
//!         .unwrap();
//!     assert_eq!(result, 1);
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
#![warn(missing_docs, unreachable_pub)]

use std::any::Any;
use std::fmt;
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
        if conn.conn.is_poisoned() {
            return Err(RecycleError::Message(
                "Mutex is poisoned. Connection is considered unusable.".to_string(),
            ));
        }
        let recycle_count = self.recycle_count.fetch_add(1, Ordering::Relaxed);
        let n: usize = conn
            .interact(move |conn| conn.query_row("SELECT $1", [recycle_count], |row| row.get(0)))
            .await
            .map_err(|e| RecycleError::Message(format!("{}", e)))?;
        if n == recycle_count {
            Ok(())
        } else {
            Err(RecycleError::Message(String::from(
                "Recycle count mismatch",
            )))
        }
    }
}

/// This error is returned when the call to [`Connection::interact`]
/// fails.
#[derive(Debug)]
pub enum InteractError {
    /// The provided callback panicked
    Panic(Box<dyn Any + Send + 'static>),
    /// The callback was aborted
    Aborted,
    /// rusqlite returned an error
    Rusqlite(rusqlite::Error),
}

impl fmt::Display for InteractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panic(_) => write!(f, "Panic"),
            Self::Aborted => write!(f, "Aborted"),
            Self::Rusqlite(e) => write!(f, "Rusqlite error: {}", e),
        }
    }
}

impl std::error::Error for InteractError {}

/// A wrapper for `rusqlite::Connection` which provides indirect
/// access to it via the `interact` function.
pub struct ConnectionWrapper {
    conn: Arc<Mutex<Option<rusqlite::Connection>>>,
}

impl ConnectionWrapper {
    async fn open(config: &Config) -> Result<Self, Error> {
        let config = config.clone();
        let conn = tokio::task::spawn_blocking(move || rusqlite::Connection::open(config.path))
            .await
            .unwrap()?;
        Ok(ConnectionWrapper {
            conn: Arc::new(Mutex::new(Some(conn))),
        })
    }
    /// Interact with the underlying SQLite connection. This function
    /// expects a closure that takes the connection as parameter. The
    /// closure is executed in a separate thread so that the async runtime
    /// is not blocked.
    pub async fn interact<F, R>(&self, f: F) -> Result<R, InteractError>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<R, rusqlite::Error> + Send + 'static,
        R: Send + 'static,
    {
        let arc = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let guard = arc.lock().unwrap();
            let conn = guard.as_ref().unwrap();
            f(&conn)
        })
        .await
        .map_err(|e| {
            if e.is_panic() {
                InteractError::Panic(e.into_panic())
            } else if e.is_cancelled() {
                InteractError::Aborted
            } else {
                unreachable!();
            }
        })?
        .map_err(InteractError::Rusqlite)
    }
}

impl Drop for ConnectionWrapper {
    fn drop(&mut self) {
        let arc = self.conn.clone();
        // Drop the `rusqlite::Connection` inside a `spawn_blocking`
        // as the `drop` function of it can block.
        tokio::task::spawn_blocking(move || match arc.lock() {
            Ok(mut guard) => guard.take(),
            Err(e) => e.into_inner().take(),
        });
    }
}
