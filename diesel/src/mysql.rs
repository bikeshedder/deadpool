//! Type aliases for using `deadpool-diesel` with MySQL.

/// Connection which is returned by the MySQL [`Pool`].
pub type Connection = crate::Connection<diesel::MysqlConnection>;

/// Manager which is used to create [`diesel::MysqlConnection`]s.
pub type Manager = crate::manager::Manager<diesel::MysqlConnection>;

/// Pool for using `deadpool-diesel` with MySQL.
pub type Pool = deadpool::managed::Pool<Manager>;
