use std::fmt;

/// This error is returned by the `Manager::recycle` function
#[derive(Debug)]
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

impl<E> fmt::Display for RecycleError<E>
where E: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(msg) => write!(f, "RecycleError::Message: {}", msg),
            Self::Backend(e) => write!(f, "RecycleError::Backend {}", e),
        }
    }
}

impl<E> std::error::Error for RecycleError<E> where E: std::error::Error {}

/// When `Pool::get` returns a timeout error this enum can be used
/// to figure out which step caused the timeout.
#[derive(Debug)]
pub enum TimeoutType {
    /// The timeout happened while waiting for a slot to become available
    Wait,
    /// The timeout happened while creating the object
    Create,
    /// The timeout happened while recycling an object
    Recycle,
}

/// Error structure for `Pool::get`
#[derive(Debug)]
pub enum PoolError<E> {
    /// A timeout happened
    Timeout(TimeoutType),
    /// The backend reported an error
    Backend(E),
}

impl<E> From<E> for PoolError<E> {
    fn from(e: E) -> Self {
        Self::Backend(e)
    }
}

impl<E> fmt::Display for PoolError<E>
where E: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout(tt) => write!(f, "PoolError::Timeout: {:?}", tt),
            Self::Backend(e) => write!(f, "PoolError::Backend: {}", e),
        }
    }
}

impl<E> std::error::Error for PoolError<E> where E: std::error::Error {}
