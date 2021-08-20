use std::{fmt, marker::PhantomData};

use deadpool::{
    async_trait,
    managed::{self, sync::SyncWrapper, RecycleError, RecycleResult},
    Runtime,
};

use crate::{Connection, Error};

/// [`Connection`] [`Manager`] for use with [`diesel`].
///
/// See the [`deadpool` documentation](deadpool) for usage examples.
///
/// [`Manager`]: managed::Manager
pub struct Manager<C: diesel::Connection> {
    database_url: String,
    runtime: Runtime,
    _marker: PhantomData<fn() -> C>,
}

// Implemented manually to avoid unnecessary trait bound on `C` type parameter.
impl<C: diesel::Connection> fmt::Debug for Manager<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Manager")
            .field("database_url", &self.database_url)
            .field("runtime", &self.runtime)
            .field("_marker", &self._marker)
            .finish()
    }
}

impl<C: diesel::Connection> Manager<C> {
    /// Creates a new [`Manager`] which establishes [`Connection`]s to the given
    /// `database_url`.
    #[must_use]
    pub fn new<S: Into<String>>(database_url: S, runtime: Runtime) -> Self {
        Manager {
            database_url: database_url.into(),
            runtime,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<C> managed::Manager for Manager<C>
where
    C: diesel::Connection + 'static,
{
    type Type = Connection<C>;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let database_url = self.database_url.clone();
        SyncWrapper::new(self.runtime, move || {
            C::establish(&database_url).map_err(Error::Connection)
        })
        .await
    }

    async fn recycle(&self, obj: &mut Self::Type) -> RecycleResult<Self::Error> {
        if obj.is_mutex_poisoned() {
            return Err(RecycleError::Message(
                "Mutex is poisoned. Connection is considered unusable.".into(),
            ));
        }
        obj.interact(|conn| conn.execute("SELECT 1").map_err(Error::Ping))
            .await
            .map_err(|e| RecycleError::Message(format!("Panic: {:?}", e)))
            .map(|_| ())
    }
}
