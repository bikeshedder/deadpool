use std::ops::{Deref, DerefMut};

use deadpool::managed::Object;

use crate::manager::Manager;

pub struct ConnectionWrapper<C: diesel::Connection> {
    pub(crate) conn: Option<C>,
}

unsafe impl<C: diesel::Connection + Send + 'static> Sync for ConnectionWrapper<C> {}

/// Connection which is returned by the pool. It implements
/// [diesel::Connection]() and can be used just like a normal
/// connection from diesel.
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
