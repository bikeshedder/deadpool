//! Connection pooling via r2d2.
//!
//! Note: This module requires enabling the `r2d2` feature

use std::fmt;
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::{convert::Into, ops::Deref};

pub use deadpool::managed::{Pool, PoolError};

use deadpool::managed::{RecycleError, RecycleResult};

pub type Connection<C> = deadpool::managed::Object<ConnectionWrapper<C>>;

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
pub type SqlitePool = Pool<SqliteManager>;
#[cfg(feature = "postgres")]
pub type PgPool = Pool<PgManager>;
#[cfg(feature = "mysql")]
pub type MysqlPool = Pool<MysqlManager>;

pub struct ConnectionWrapper<C: diesel::Connection> {
    conn: Option<C>,
}

unsafe impl<C: diesel::Connection + Send + 'static> Sync for ConnectionWrapper<C> {}

impl<C: diesel::Connection> Deref for ConnectionWrapper<C> {
    type Target = C;
    fn deref(&self) -> &Self::Target {
        self.conn.as_ref().unwrap()
    }
}

impl<C: diesel::Connection> DerefMut for ConnectionWrapper<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.conn.as_mut().unwrap()
    }
}

/// The error used when managing connections with `r2d2`.
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

pub struct Manager<C> {
    database_url: String,
    _marker: PhantomData<C>,
}

unsafe impl<T: Send + 'static> Sync for Manager<T> {}

impl<C> Manager<C> {
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

    pub type SqliteManager = Manager<diesel::SqliteConnection>;
    pub type SqlitePool = Pool<SqliteManager>;

    use super::*;

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
        assert_eq!("foo", query.get_result::<String>(&mut **conn).unwrap());
    }
}
