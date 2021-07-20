use std::marker::PhantomData;

use deadpool::{
    managed::{sync::SyncWrapper, RecycleError, RecycleResult},
    Runtime,
};

use crate::Error;

/// An deadpool connection manager for use with Diesel.
///
/// See the [deadpool documentation] for usage examples.
///
/// [deadpool documentation]: deadpool

pub struct Manager<C: diesel::Connection> {
    database_url: String,
    runtime: Runtime,
    _marker: PhantomData<fn() -> C>,
}

impl<C: diesel::Connection> Manager<C> {
    /// Create manager which establishes connections to the
    /// given database URL.
    pub fn new<S: Into<String>>(database_url: S, runtime: Runtime) -> Self {
        Manager {
            database_url: database_url.into(),
            runtime,
            _marker: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<C> deadpool::managed::Manager for Manager<C>
where
    C: diesel::Connection + 'static,
{
    type Type = SyncWrapper<C, Error>;
    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let database_url = self.database_url.clone();
        SyncWrapper::new(self.runtime.clone(), move || {
            C::establish(&database_url).map_err(Error::ConnectionError)
        })
        .await
    }

    async fn recycle(&self, obj: &mut Self::Type) -> RecycleResult<Self::Error> {
        if obj.is_mutex_poisoned() {
            return Err(RecycleError::Message(
                "Mutex is poisoned. Connection is considered unusable.".to_string(),
            ));
        }
        obj.interact(|conn| conn.execute("SELECT 1").map_err(Error::QueryError))
            .await
            .map_err(|e| RecycleError::Message(format!("Panic: {:?}", e)))
            .map(|_| ())
    }
}
