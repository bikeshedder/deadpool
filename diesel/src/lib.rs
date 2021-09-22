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

mod error;
mod manager;

#[cfg(feature = "mysql")]
#[cfg_attr(docsrs, doc(cfg(feature = "mysql")))]
pub mod mysql;
#[cfg(feature = "postgres")]
#[cfg_attr(docsrs, doc(cfg(feature = "postgres")))]
pub mod postgres;
#[cfg(feature = "sqlite")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]
pub mod sqlite;

use deadpool::managed;

pub use deadpool::{
    managed::{sync::SyncGuard, Pool, PoolConfig, Timeouts},
    Runtime,
};

pub use self::{error::Error, manager::Manager};

/// Type alias for using [`deadpool::managed::PoolError`] with [`diesel`].
pub type PoolError = managed::PoolError<Error>;

/// Connection which is returned by the [`Pool`].
pub type Connection<C> = managed::sync::SyncWrapper<C, Error>;
