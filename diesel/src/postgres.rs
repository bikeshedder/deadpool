//! Type aliases for using `deadpool-diesel` with PostgreSQL.

/// Manager for PostgreSQL connections
pub type Manager = crate::Manager<diesel::PgConnection>;

deadpool::managed_reexports!(
    "diesel",
    Manager,
    deadpool::managed::Object<Manager>,
    diesel::ConnectionError,
    std::convert::Infallible
);

/// Type alias for [`Object`]
pub type Connection = Object;
