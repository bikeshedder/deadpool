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

use std::sync::atomic::{AtomicUsize, Ordering};

use deadpool::{
    async_trait,
    managed::{self, sync::SyncWrapper, RecycleError},
};

pub use deadpool::managed::sync::reexports::*;
pub use rusqlite;

pub use self::config::Config;

/// Type alias for using [`deadpool::managed::Pool`] with [`rusqlite`].
pub type Pool = managed::Pool<Manager>;

/// Type alias for using [`deadpool::managed::PoolError`] with [`rusqlite`].
pub type PoolError = managed::PoolError<rusqlite::Error>;

/// Type alias for using [`deadpool::managed::Object`] with [`rusqlite`].
pub type Connection = managed::Object<Manager>;

/// [`Manager`] for creating and recycling SQLite [`Connection`]s.
///
/// [`Manager`]: managed::Manager
#[derive(Debug)]
pub struct Manager {
    config: Config,
    recycle_count: AtomicUsize,
    runtime: Runtime,
}

impl Manager {
    /// Creates a new [`Manager`] using the given [`Config`] backed by the
    /// specified [`Runtime`].
    #[must_use]
    pub fn from_config(config: &Config, runtime: Runtime) -> Self {
        Self {
            config: config.clone(),
            recycle_count: AtomicUsize::new(0),
            runtime,
        }
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = SyncWrapper<rusqlite::Connection, rusqlite::Error>;
    type Error = rusqlite::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let path = self.config.path.clone();
        SyncWrapper::new(self.runtime, move || rusqlite::Connection::open(path)).await
    }

    async fn recycle(&self, conn: &mut Self::Type) -> managed::RecycleResult<Self::Error> {
        if conn.is_mutex_poisoned() {
            return Err(RecycleError::Message(
                "Mutex is poisoned. Connection is considered unusable.".into(),
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
            Err(RecycleError::Message("Recycle count mismatch".into()))
        }
    }
}
