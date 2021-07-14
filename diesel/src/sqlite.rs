//! Type aliases for using `deadpool-diesel` with SQLite.

/// Connection which is returned by the SQLite pool
pub type Connection = crate::connection::Connection<diesel::SqliteConnection>;

/// Manager which is used to create SQLite connections
pub type Manager = crate::manager::Manager<diesel::SqliteConnection>;

/// Pool for using `deadpool-diesel` with SQLite
pub type Pool = deadpool::managed::Pool<Manager, Connection>;
