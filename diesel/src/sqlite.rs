//! Type aliases for using `deadpool-diesel` with SQLite.

/// Connection which is returned by the SQLite [`Pool`].
pub type Connection = crate::Connection<diesel::SqliteConnection>;

/// Manager which is used to create [`diesel::SqliteConnection`]s.
pub type Manager = crate::manager::Manager<diesel::SqliteConnection>;

/// Pool for using `deadpool-diesel` with SQLite.
pub type Pool = deadpool::managed::Pool<Manager>;
