//! # Deadpool for PostgreSQL [![Latest Version](https://img.shields.io/crates/v/deadpool-postgres.svg)](https://crates.io/crates/deadpool-postgres)
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
//! TODO
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

pub use deadpool::managed::{Pool, PoolError};

mod connection;
mod error;
mod manager;

pub use connection::Connection;
pub use error::Error;
pub use manager::Manager;

#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;
