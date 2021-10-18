//! Type aliases for using `deadpool-diesel` with SQLite.

/// Manager for SQLite connections
pub type Manager = crate::Manager<diesel::SqliteConnection>;

pub use deadpool::managed::sync::reexports::*;
deadpool::managed_reexports!(
    "diesel",
    Manager,
    deadpool::managed::Object<Manager>,
    diesel::ConnectionError,
    std::convert::Infallible
);

/// Type alias for [`Object`]
pub type Connection = Object;
