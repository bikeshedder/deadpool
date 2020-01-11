//! Deadpool is a dead simple async pool for connections and objects
//! of any type.
//!
//! It provides two implementations:
//! - A `managed` one which requires a `Manager` trait which is responsible
//!   for creating and recycling objects as they are needed.
//! - An `unmanaged` one which requires the objects to be created upfront and
//!   is a lot simpler, too.
#![warn(missing_docs)]

pub mod managed;
pub mod unmanaged;

#[derive(Debug)]
/// The current pool status.
pub struct Status {
    /// The size of the pool
    pub size: usize,
    /// The number of available objects in the pool. If there are no
    /// objects in the pool this number can become negative and stores the
    /// number of futures waiting for an object.
    pub available: isize,
}

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::Object` directly"
)]
/// This struct was moved to `deadpool::managed::Object`.
pub type Object<T, E> = managed::Object<T, E>;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::Pool` directly"
)]
/// This struct was moved to `deadpool::managed::Pool`.
pub type Pool<T, E> = managed::Pool<T, E>;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::PoolConfig` directly"
)]
/// This struct was moved to`deadpool::managed::PoolConfig`.
pub type PoolConfig = managed::PoolConfig;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::Timeouts` directly"
)]
/// This struct was moved to `deadpool::managed::Timeouts`.
pub type Timeouts = managed::Timeouts;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::PoolError` directly"
)]
/// This enum was moved to `deadpool::managed::PoolError`.
pub type PoolError<E> = managed::PoolError<E>;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::RecycleError` directly"
)]
/// This enum was moved to `deadpool::managed::RecycleError`.
pub type RecycleError<E> = managed::RecycleError<E>;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::TimeoutType` directly"
)]
/// This enum was moved to `deadpool::managed::TimeoutType`.
pub type TimeoutType = managed::TimeoutType;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::Manager` directly"
)]
/// This trait was moved to `deadpool::managed::Manager`.
pub type Manager<T, E> = dyn managed::Manager<T, E>;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::RecycleResult` directly"
)]
/// This type was moved to `deadpool::managed::RecycleResult`.
pub type RecycleResult<E> = managed::RecycleResult<E>;
