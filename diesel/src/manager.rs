use std::marker::PhantomData;

use deadpool::managed::{RecycleError, RecycleResult};

use crate::{connection::ConnectionWrapper, Error};

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
