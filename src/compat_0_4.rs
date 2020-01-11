#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::Object` directly"
)]
/// This struct was moved to `deadpool::managed::Object`.
pub type Object<T, E> = crate::managed::Object<T, E>;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::Pool` directly"
)]
/// This struct was moved to `deadpool::managed::Pool`.
pub type Pool<T, E> = crate::managed::Pool<T, E>;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::PoolConfig` directly"
)]
/// This struct was moved to`deadpool::managed::PoolConfig`.
pub type PoolConfig = crate::managed::PoolConfig;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::Timeouts` directly"
)]
/// This struct was moved to `deadpool::managed::Timeouts`.
pub type Timeouts = crate::managed::Timeouts;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::PoolError` directly"
)]
/// This enum was moved to `deadpool::managed::PoolError`.
pub type PoolError<E> = crate::managed::PoolError<E>;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::RecycleError` directly"
)]
/// This enum was moved to `deadpool::managed::RecycleError`.
pub type RecycleError<E> = crate::managed::RecycleError<E>;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::TimeoutType` directly"
)]
/// This enum was moved to `deadpool::managed::TimeoutType`.
pub type TimeoutType = crate::managed::TimeoutType;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::Manager` directly"
)]
/// This trait was moved to `deadpool::managed::Manager`.
pub type Manager<T, E> = dyn crate::managed::Manager<T, E>;

#[deprecated(
    since = "0.5.0",
    note = "Please use `deadpool::managed::RecycleResult` directly"
)]
/// This type was moved to `deadpool::managed::RecycleResult`.
pub type RecycleResult<E> = crate::managed::RecycleResult<E>;
