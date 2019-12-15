use tokio::time::Elapsed;

/// This error is returned by the `Manager::recycle` function
pub enum RecycleError<E> {
    /// Recycling failed for some other reason
    Message(String),
    /// The error was caused by the backend
    Backend(E),
}

impl<E> From<E> for RecycleError<E> {
    fn from(e: E) -> Self {
        Self::Backend(e)
    }
}

/// When `Pool::get` returns a timeout error this enum can be used
/// to figure out which step caused the timeout.
#[derive(Debug)]
pub enum TimeoutType {
    /// The timeout happened while creating the object
    Create,
    /// The timeout happened while waiting for an object to become available
    Wait,
    /// The timeout happened while recycling an object
    Recycle,
}

/// Error structure for `Pool::get`
#[derive(Debug)]
pub enum PoolError<E> {
    /// A timeout happened
    Timeout(TimeoutType, Elapsed),
    /// The backend reported an error
    Backend(E),
}

impl<E> From<E> for PoolError<E> {
    fn from(e: E) -> Self {
        Self::Backend(e)
    }
}
