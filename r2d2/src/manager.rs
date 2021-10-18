use std::{fmt, sync::Arc};

use deadpool::{
    async_trait,
    managed::{self, sync::SyncWrapper, RecycleError, RecycleResult},
    Runtime,
};

/// [`Connection`] [`Manager`] for use with [`r2d2`] managers.
///
/// See the [`deadpool` documentation](deadpool) for usage examples.
///
/// [`Manager`]: managed::Manager
pub struct Manager<M: r2d2::ManageConnection> {
    r2d2_manager: Arc<M>,
    runtime: Runtime,
}

// Implemented manually to avoid unnecessary trait bound on `C` type parameter.
impl<M: r2d2::ManageConnection> fmt::Debug for Manager<M>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Manager")
            .field("r2d2_manager", &self.r2d2_manager)
            .field("runtime", &self.runtime)
            .finish()
    }
}

impl<M: r2d2::ManageConnection> Manager<M> {
    /// Creates a new [`Manager`] which establishes [`Connection`]s to the given
    /// `database_url`.
    #[must_use]
    pub fn new(r2d2_manager: M, runtime: Runtime) -> Self {
        Manager {
            runtime,
            r2d2_manager: Arc::new(r2d2_manager),
        }
    }
}

#[async_trait]
impl<M: r2d2::ManageConnection> managed::Manager for Manager<M>
where
    M::Error: Send,
{
    type Type = SyncWrapper<M::Connection, M::Error>;
    type Error = M::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let r2d2_manager = self.r2d2_manager.clone();
        SyncWrapper::new(self.runtime, move || r2d2_manager.connect()).await
    }

    async fn recycle(&self, obj: &mut Self::Type) -> RecycleResult<Self::Error> {
        if obj.is_mutex_poisoned() {
            return Err(RecycleError::Message(
                "Mutex is poisoned. Connection is considered unusable.".into(),
            ));
        }
        let r2d2_manager = self.r2d2_manager.clone();
        obj.interact::<_, RecycleResult<Self::Error>>(move |obj| {
            if r2d2_manager.has_broken(obj) {
                Ok(Err(RecycleError::Message("Connection is broken".into())))
            } else {
                Ok(r2d2_manager.is_valid(obj).map_err(RecycleError::Backend))
            }
        })
        .await
        .map_err(|e| RecycleError::Message(format!("Interaction failed: {}", e)))?
    }
}
