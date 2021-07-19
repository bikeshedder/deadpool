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
//! | `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
//! | `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate | `deadpool/rt_async-std_1` | no |
//!
//! ## Example
//!
//! ```rust
//! use deadpool_sqlite::{Config, Runtime};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut cfg = Config::new("db.sqlite3");
//!     let pool = cfg.create_pool(Runtime::Tokio1).unwrap();
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

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use deadpool::managed::sync::SyncWrapper;
use deadpool::managed::RecycleError;
use rusqlite::Error;

mod config;
pub use config::Config;

pub use deadpool::managed::sync::InteractError;
pub use deadpool::managed::PoolConfig;
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
    runtime: Runtime,
}

impl Manager {
    /// Create manager using a `deadpool_sqlite::Config`
    pub fn from_config(config: &Config, runtime: Runtime) -> Self {
        Self {
            config: config.clone(),
            recycle_count: AtomicUsize::new(0),
            runtime,
        }
    }
}

#[async_trait::async_trait]
impl deadpool::managed::Manager for Manager {
    type Type = SyncWrapper<rusqlite::Connection, rusqlite::Error>;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let path = self.config.path.clone();
        SyncWrapper::new(self.runtime.clone(), move || {
            rusqlite::Connection::open(path)
        })
        .await
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        if conn.is_mutex_poisoned() {
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
