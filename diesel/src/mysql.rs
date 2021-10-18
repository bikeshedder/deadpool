//! Type aliases for using `deadpool-diesel` with MySQL.

/// Manager for MySQL connections
pub type Manager = crate::Manager<diesel::MysqlConnection>;

deadpool::managed_reexports!(
    "diesel",
    Manager,
    deadpool::managed::Object<Manager>,
    diesel::ConnectionError,
    std::convert::Infallible
);

/// Type alias for [`Object`]
pub type Connection = Object;
