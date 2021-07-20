//! Type aliases for using `deadpool-diesel` with PostgreSQL.

/// Connection which is returned by the PostgreSQL pool
pub type Connection = crate::Connection<diesel::PgConnection>;

/// Manager which is used to create PostgreSQL connections
pub type Manager = crate::manager::Manager<diesel::PgConnection>;

/// Pool for using `deadpool-diesel` with PostgreSQL
pub type Pool = deadpool::managed::Pool<Manager>;
