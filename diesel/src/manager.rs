use std::{fmt, marker::PhantomData};

use deadpool::{
    async_trait,
    managed::{self, Metrics, RecycleError, RecycleResult},
    Runtime,
};
use deadpool_sync::SyncWrapper;
use diesel::{
    backend::Backend,
    query_builder::{Query, QueryFragment, QueryId},
    QueryResult, RunQueryDsl,
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
            C::establish(&database_url).map_err(Into::into)
        })
        .await
    }

    async fn recycle(&self, obj: &mut Self::Type, _: &Metrics) -> RecycleResult<Self::Error> {
        if obj.is_mutex_poisoned() {
            return Err(RecycleError::StaticMessage(
                "Mutex is poisoned. Connection is considered unusable.",
            ));
        }
        obj.interact(|conn| CheckConnectionQuery.execute(conn).map_err(Error::Ping))
            .await
            .map_err(|e| RecycleError::Message(format!("Panic: {:?}", e)))
            .and_then(|r| r.map_err(RecycleError::Backend))
            .map(|_| ())
    }
}

// The `CheckConnectionQuery` is a 1:1 copy of the code found in
// the `diesel::r2d2` module:
// https://github.com/diesel-rs/diesel/blob/master/diesel/src/r2d2.rs

#[derive(QueryId)]
struct CheckConnectionQuery;

impl<DB> QueryFragment<DB> for CheckConnectionQuery
where
    DB: Backend,
{
    fn walk_ast<'b>(
        &'b self,
        mut pass: diesel::query_builder::AstPass<'_, 'b, DB>,
    ) -> QueryResult<()> {
        pass.push_sql("SELECT 1");
        Ok(())
    }
}

impl Query for CheckConnectionQuery {
    type SqlType = diesel::sql_types::Integer;
}

impl<C> RunQueryDsl<C> for CheckConnectionQuery {}
