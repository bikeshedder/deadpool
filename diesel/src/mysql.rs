//! Type aliases for using `deadpool-diesel` with MySQL.

/// Connection which is returned by the MySQL pool
pub type Connection = crate::connection::Connection<diesel::MysqlConnection>;

/// Manager which is used to create MySQL connections
pub type Manager = crate::manager::Manager<diesel::MysqlConnection>;

/// Pool for using `deadpool-diesel` with MySQL
pub type Pool = deadpool::managed::Pool<Manager, Connection>;
