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
//! | `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |
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

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use deadpool::managed::Pool;

    use super::*;

    // These type aliases are repeated here as there is no
    // way to enable a crate local feature for the test profile:
    // https://github.com/rust-lang/cargo/issues/2911
    type SqliteConnection = Connection<diesel::SqliteConnection>;
    type SqliteManager = Manager<diesel::SqliteConnection>;
    type SqlitePool = Pool<SqliteManager, SqliteConnection>;

    fn create_pool(max_size: usize) -> SqlitePool {
        let manager = SqliteManager::new(":memory:");
        let pool = SqlitePool::new(manager, max_size);
        pool
    }

    #[tokio::test]
    async fn establish_basic_connection() {
        let pool = create_pool(2);

        let (s1, mut r1) = mpsc::channel(1);
        let (s2, mut r2) = mpsc::channel(1);

        let pool1 = pool.clone();
        let t1 = tokio::spawn(async move {
            let conn = pool1.get().await.unwrap();
            s1.send(()).await.unwrap();
            r2.recv().await.unwrap();
            drop(conn)
        });

        let pool2 = pool.clone();
        let t2 = tokio::spawn(async move {
            let conn = pool2.get().await.unwrap();
            s2.send(()).await.unwrap();
            r1.recv().await.unwrap();
            drop(conn)
        });

        t1.await.unwrap();
        t2.await.unwrap();

        pool.get().await.unwrap();
    }

    #[tokio::test]
    async fn pooled_connection_impls_connection() {
        use diesel::prelude::*;
        use diesel::select;
        use diesel::sql_types::Text;

        let pool = create_pool(1);
        let mut conn = pool.get().await.unwrap();

        let query = select("foo".into_sql::<Text>());
        assert_eq!("foo", query.get_result::<String>(&mut conn).unwrap());
    }
}
