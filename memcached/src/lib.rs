//! # Deadpool for Memcache
//!
//! Deadpool is a dead simple async pool for connections and objects of any type.
//!
//! This crate implements a [`deadpool`](https://crates.io/crates/deadpool) manager for
//! [`async-memcached`](https://crates.io/crates/async-memcached).  We specifically force users to
//! connect via TCP as there is no existing mechanism to parameterize how to deal with different
//! unerlying connection types at the moment.
#![deny(warnings, missing_docs)]
use async_memcached::{Client, Error};
use async_trait::async_trait;

/// A type alias for using `deadpool::Pool` with `async-memcached`
pub type Pool = deadpool::managed::Pool<Client, Error>;

/// A type alias for using `deadpool::PoolError` with `async-memcached`
pub type PoolError = deadpool::managed::PoolError<Error>;

/// A type alias for using `deadpool::Object` with `async-memcached`
pub type Connection = deadpool::managed::Object<Client, Error>;

type RecycleResult = deadpool::managed::RecycleResult<Error>;

/// The manager for creating and recyling memcache connections
pub struct Manager {
    addr: String,
}

impl Manager {
    /// Create a new manager for the given address.
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[async_trait]
impl deadpool::managed::Manager<Client, Error> for Manager {
    async fn create(&self) -> Result<Client, Error> {
        Client::new(&self.addr).await
    }

    async fn recycle(&self, conn: &mut Client) -> RecycleResult {
        match conn.version().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
