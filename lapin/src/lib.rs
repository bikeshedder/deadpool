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

use deadpool::{async_trait, managed};
use lapin::{ConnectionProperties, Error};

pub use deadpool::managed::reexports::*;
pub use lapin;

pub use self::config::{BuildError, Config};

/// Type alias for using [`deadpool::managed::Pool`] with [`lapin`].
pub type Pool = managed::Pool<Manager>;

/// Type alias for using [`deadpool::managed::PoolError`] with [`lapin`].
pub type PoolError = managed::PoolError<Error>;

/// Type alias for using [`deadpool::managed::Object`] with [`lapin`].
pub type Connection = managed::Object<Manager>;

type RecycleResult = managed::RecycleResult<Error>;
type RecycleError = managed::RecycleError<Error>;

/// [`Manager`] for creating and recycling [`lapin::Connection`].
///
/// [`Manager`]: managed::Manager
#[derive(Debug)]
pub struct Manager {
    addr: String,
    connection_properties: ConnectionProperties,
}

impl Manager {
    /// Creates a new [`Manager`] using the given AMQP address and
    /// [`lapin::ConnectionProperties`].
    #[must_use]
    pub fn new<S: Into<String>>(addr: S, connection_properties: ConnectionProperties) -> Self {
        Self {
            addr: addr.into(),
            connection_properties,
        }
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = lapin::Connection;
    type Error = Error;

    async fn create(&self) -> Result<lapin::Connection, Error> {
        let conn =
            lapin::Connection::connect(self.addr.as_str(), self.connection_properties.clone())
                .await?;
        Ok(conn)
    }

    async fn recycle(&self, conn: &mut lapin::Connection) -> RecycleResult {
        match conn.status().state() {
            lapin::ConnectionState::Connected => Ok(()),
            other_state => Err(RecycleError::Message(format!(
                "lapin connection is in state: {:?}",
                other_state
            ))),
        }
    }
}
