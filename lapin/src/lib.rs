//! Deadpool simple async pool for AMQP connections.
//!
//! This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
//! manager for [`lapin`](https://crates.io/crates/lapin).
//!
//! You should not need to use `deadpool` directly. Use the `Pool` type
//! provided by this crate instead.
//!
//! # Example
//!
//! ```rust
//! use std::env;
//!
//! use deadpool_lapin::{Manager, Pool};
//! use lapin::{
//!     ConnectionProperties,
//!     options::BasicPublishOptions,
//!     BasicProperties
//! };
//!
//! #[tokio::main]
//! async fn main() {
//!     let addr = std::env::var("AMQP_ADDR").unwrap_or_else(
//!         |_| "amqp://127.0.0.1:5672/%2f".into());
//!     let mgr = Manager::new(addr, ConnectionProperties::default());
//!     let pool = Pool::new(mgr, 16);
//!     for i in 1..10 {
//!         let mut connection = pool.get().await.unwrap();
//!         let channel = connection.create_channel().await.unwrap();
//!         channel.basic_publish(
//!             "",
//!             "hello",
//!             BasicPublishOptions::default(),
//!             b"hello from deadpool".to_vec(),
//!             BasicProperties::default()
//!         ).await.unwrap();
//!     }
//! }
//! ```
#![warn(missing_docs)]

use async_trait::async_trait;
use lapin::{ConnectionProperties, Error};

/// A type alias for using `deadpool::Pool` with `lapin`
pub type Pool = deadpool::managed::Pool<lapin::Connection, Error>;

/// A type alias for using `deadpool::PoolError` with `lapin`
pub type PoolError = deadpool::managed::PoolError<Error>;

/// A type alias for using `deadpool::Object` with `lapin`
pub type Connection = deadpool::managed::Object<lapin::Connection, Error>;

type RecycleResult = deadpool::managed::RecycleResult<Error>;
type RecycleError = deadpool::managed::RecycleError<Error>;

/// The manager for creating and recyling lapin connections
pub struct Manager {
    addr: String,
    connection_properties: ConnectionProperties,
}

impl Manager {
    /// Create manager using `PgConfig` and a `TlsConnector`
    pub fn new(addr: String, connection_properties: ConnectionProperties) -> Self {
        Self {
            addr: addr,
            connection_properties: connection_properties,
        }
    }
}

#[async_trait]
impl deadpool::managed::Manager<lapin::Connection, Error> for Manager {
    async fn create(&self) -> Result<lapin::Connection, Error> {
        let connection =
            lapin::Connection::connect(self.addr.as_str(), self.connection_properties.clone())
                .await?;
        Ok(connection)
    }
    async fn recycle(&self, connection: &mut lapin::Connection) -> RecycleResult {
        match connection.status().state() {
            lapin::ConnectionState::Connected => Ok(()),
            other_state => Err(RecycleError::Message(format!(
                "lapin connection is in state: {:?}",
                other_state
            ))),
        }
    }
}
