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

use std::fmt;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::{convert::Into, ops::Deref};

pub use deadpool::managed::{Pool, PoolError};

use deadpool::managed::{Object, RecycleError, RecycleResult};

#[cfg(feature = "sqlite")]
pub type SqliteConnection = Connection<diesel::SqliteConnection>;
#[cfg(feature = "postgres")]
pub type PgConnection = Connection<diesel::PgConnection>;
#[cfg(feature = "mysql")]
pub type MysqlConnection = Connection<diesel::MysqlConnection>;

#[cfg(feature = "sqlite")]
pub type SqliteManager = Manager<diesel::SqliteConnection>;
#[cfg(feature = "postgres")]
pub type PgManager = Manager<diesel::PgConnection>;
#[cfg(feature = "mysql")]
pub type MysqlManager = Manager<diesel::MysqlConnection>;

#[cfg(feature = "sqlite")]
pub type SqlitePool = Pool<SqliteManager, SqliteConnection>;
#[cfg(feature = "postgres")]
pub type PgPool = Pool<PgManager, PgConnection>;
#[cfg(feature = "mysql")]
pub type MysqlPool = Pool<MysqlManager, MysqlConnection>;

pub struct ConnectionWrapper<C: diesel::Connection> {
    conn: Option<C>,
}

unsafe impl<C: diesel::Connection + Send + 'static> Sync for ConnectionWrapper<C> {}

pub struct Connection<C: diesel::Connection + 'static> {
    obj: deadpool::managed::Object<Manager<C>>,
}

impl<C: diesel::Connection> Deref for Connection<C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        self.obj.conn.as_ref().unwrap()
    }
}

impl<C: diesel::Connection> From<Object<Manager<C>>> for Connection<C> {
    fn from(obj: Object<Manager<C>>) -> Self {
        Self { obj }
    }
}

impl<C: diesel::Connection> DerefMut for Connection<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.obj.conn.as_mut().unwrap()
    }
}

impl<C: diesel::Connection> diesel::connection::SimpleConnection for Connection<C> {
    fn batch_execute(&self, query: &str) -> diesel::QueryResult<()> {
        (**self).batch_execute(query)
    }
}

impl<C> diesel::Connection for Connection<C>
where
    C: diesel::Connection<TransactionManager = diesel::connection::AnsiTransactionManager>
        + Send
        + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    type Backend = C::Backend;
    type TransactionManager = C::TransactionManager;

    fn establish(_: &str) -> diesel::ConnectionResult<Self> {
        Err(diesel::ConnectionError::BadConnection(String::from(
            "Cannot directly establish a pooled connection",
        )))
    }

    fn execute(&self, query: &str) -> diesel::QueryResult<usize> {
        (**self).execute(query)
    }

    fn query_by_index<T, U>(&self, source: T) -> diesel::QueryResult<Vec<U>>
    where
        T: diesel::query_builder::AsQuery,
        T::Query:
            diesel::query_builder::QueryFragment<Self::Backend> + diesel::query_builder::QueryId,
        Self::Backend: diesel::types::HasSqlType<T::SqlType>,
        U: diesel::Queryable<T::SqlType, Self::Backend>,
    {
        (**self).query_by_index(source)
    }

    fn query_by_name<T, U>(&self, source: &T) -> diesel::QueryResult<Vec<U>>
    where
        T: diesel::query_builder::QueryFragment<Self::Backend> + diesel::query_builder::QueryId,
        U: diesel::deserialize::QueryableByName<Self::Backend>,
    {
        (**self).query_by_name(source)
    }

    fn execute_returning_count<T>(&self, source: &T) -> diesel::QueryResult<usize>
    where
        T: diesel::query_builder::QueryFragment<Self::Backend> + diesel::query_builder::QueryId,
    {
        (**self).execute_returning_count(source)
    }

    fn transaction_manager(&self) -> &Self::TransactionManager {
        (**self).transaction_manager()
    }
}

/// The error used when managing connections with `deadpool`.
#[derive(Debug)]
pub enum Error {
    /// An error occurred establishing the connection
    ConnectionError(diesel::ConnectionError),

    /// An error occurred pinging the database
    QueryError(diesel::result::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ConnectionError(ref e) => e.fmt(f),
            Error::QueryError(ref e) => e.fmt(f),
        }
    }
}

impl ::std::error::Error for Error {}

/// An deadpool connection manager for use with Diesel.
///
/// See the [deadpool documentation] for usage examples.
///
/// [deadpool documentation]: deadpool

pub struct Manager<C: diesel::Connection> {
    database_url: String,
    _marker: PhantomData<C>,
}

unsafe impl<C: diesel::Connection> Sync for Manager<C> {}

impl<C: diesel::Connection> Manager<C> {
    /// Create manager which establishes connections to the
    /// given database URL.
    pub fn new<S: Into<String>>(database_url: S) -> Self {
        Manager {
            database_url: database_url.into(),
            _marker: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<C> deadpool::managed::Manager for Manager<C>
where
    C: diesel::Connection + 'static,
{
    type Type = ConnectionWrapper<C>;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let database_url = self.database_url.clone();
        tokio::task::spawn_blocking(move || C::establish(&database_url))
            .await
            .unwrap()
            .map_err(Error::ConnectionError)
            .map(|conn| ConnectionWrapper { conn: Some(conn) })
    }

    async fn recycle(&self, obj: &mut Self::Type) -> RecycleResult<Self::Error> {
        let conn = obj.conn.take().ok_or(RecycleError::Message(String::from("Connection is gone. This is probably caused by a previous unsuccessful recycle attempt.")))?;
        let conn = tokio::task::spawn_blocking(move || {
            conn.execute("SELECT 1")
                .map_err(Error::QueryError)
                .map_err(RecycleError::Backend)
                .and(Ok(conn))
        })
        .await
        .unwrap()?;
        obj.conn.replace(conn);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use deadpool::managed::Pool;

    use super::*;

    pub type SqliteConnection = Connection<diesel::SqliteConnection>;
    pub type SqliteManager = Manager<diesel::SqliteConnection>;
    pub type SqlitePool = Pool<SqliteManager, SqliteConnection>;

    #[tokio::test]
    async fn establish_basic_connection() {
        let manager = SqliteManager::new(":memory:");
        let pool = Pool::<SqliteManager>::new(manager, 2);

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

        let manager = SqliteManager::new(":memory:");
        let pool = SqlitePool::new(manager, 1);
        let mut conn = pool.get().await.unwrap();

        let query = select("foo".into_sql::<Text>());
        assert_eq!("foo", query.get_result::<String>(&mut conn).unwrap());
    }
}
