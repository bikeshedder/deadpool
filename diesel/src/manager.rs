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
    recycle_check: fn(&mut C) -> Result<(), Error>,
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

impl<C> Manager<C>
where
    C: diesel::Connection,
{
    /// Creates a new [`Manager`] which establishes [`Connection`]s to the given
    /// `database_url`.
    #[must_use]
    pub fn new<S: Into<String>>(database_url: S, runtime: Runtime) -> Self {
        Manager {
            database_url: database_url.into(),
            runtime,
            recycle_check: Self::default_recycle_check,
            _marker: PhantomData,
        }
    }

    /// This function set a custom check function for the checks performed
    /// when a connection is returned to the pool.
    ///
    /// It's useful to configure a custom check function here for the following cases:
    ///
    /// * Usage of custom backand that requires a different ping query
    /// * Customizing the ping query
    /// * Disabling the ping query
    ///
    /// By default the `Self::default_recycle_check` function is used. It will check
    /// whether the transaction manager is considered to be broken. If that's not the
    /// case it will execute a `SELECT 1` ping query to see whether the connection
    /// is still alive.
    pub fn with_custom_recycle_check(mut self, check: fn(&mut C) -> Result<(), Error>) -> Self {
        self.recycle_check = check;
        self
    }

    /// The default recycle check function used by `Manager`
    pub fn default_recycle_check(conn: &mut C) -> Result<(), Error> {
        use diesel::connection::TransactionManager;

        if C::TransactionManager::is_broken_transaction_manager(conn) {
            Err(Error::BrokenTransactionManger)
        } else {
            CheckConnectionQuery
                .execute(conn)
                .map_err(Error::Ping)
                .map(|_| ())
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
        obj.interact(self.recycle_check)
            .await
            .map_err(|e| RecycleError::Message(format!("Panic: {:?}", e)))
            .and_then(|r| r.map_err(RecycleError::Backend))
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
