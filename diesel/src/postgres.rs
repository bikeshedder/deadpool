//! Type aliases for using `deadpool-diesel` with PostgreSQL.

/// Connection which is returned by the PostgreSQL [`Pool`].
pub type Connection = crate::Connection<diesel::PgConnection>;

/// Manager which is used to create [`diesel::PgConnection`]s.
pub type Manager = crate::manager::Manager<diesel::PgConnection>;

/// Pool for using `deadpool-diesel` with PostgreSQL.
pub type Pool = deadpool::managed::Pool<Manager>;
