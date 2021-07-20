//! # Deadpool for Diesel [![Latest Version](https://img.shields.io/crates/v/deadpool-diesel.svg)](https://crates.io/crates/deadpool-diesel)
//!
//! Deadpool is a dead simple async pool for connections and objects
//! of any type.
//!
//! This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
//! manager for [`diesel`](https://crates.io/crates/diesel) connections.
//!
//! ## Features
//!
//! | Feature | Description | Extra dependencies | Default |
//! | ------- | ----------- | ------------------ | ------- |
//! | `sqlite` | Enable `sqlite` feature in `diesel` crate | `diesel/sqlite` | no |
//! | `postgres` | Enable `postgres` feature in `diesel` crate | `diesel/postgres` | no |
//! | `mysql` | Enable `mysql` feature in `diesel` crate | `diesel/mysql` | no |
//! | `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
//! | `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate | `deadpool/rt_async-std_1` | no |
//!
//! ## Example
//!
//! ```rust
//! use deadpool_diesel::{Runtime, sqlite::{Manager, Pool}};
//! use diesel::{prelude::*, select, sql_types::Text};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = Manager::new(":memory:", Runtime::Tokio1);
//!     let pool = Pool::builder(manager)
//!         .max_size(8)
//!         .build()
//!         .unwrap();
//!     let conn = pool.get().await?;
//!     let result = conn.interact(|conn| {
//!         let query = select("Hello world!".into_sql::<Text>());
//!         query.get_result::<String>(conn)
//!             .map_err(Into::into)
//!     }).await.unwrap();
//!     assert!(result == "Hello world!");
//!     Ok(())
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

mod error;
mod manager;

pub use error::Error;
pub use manager::Manager;

#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use deadpool::managed::{Pool, PoolConfig, Timeouts};
pub use deadpool::Runtime;

/// A type alias for using `deadpool::PoolError` with `diesel`
pub type PoolError = deadpool::managed::PoolError<Error>;

/// A type alias for using `deadpool::managed::sync::SyncWrapper` with `diesel`
pub type Connection<C> = deadpool::managed::sync::SyncWrapper<C, Error>;
