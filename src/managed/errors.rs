use std::fmt;

use super::hooks::HookError;

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
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(msg) => write!(f, "An error occured while recycling an object: {}", msg),
            Self::Backend(e) => write!(f, "An error occured while recycling an object: {}", e),
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
    /// The pool has been closed
    Closed,
    /// No runtime specified
    NoRuntimeSpecified,
    /// A post_create hook reported an error
    PostCreateHook(HookError<E>),
    /// A post_recycle hook reported an error
    PostRecycleHook(HookError<E>),
}

impl<E> From<E> for PoolError<E> {
    fn from(e: E) -> Self {
        Self::Backend(e)
    }
}

impl<E> fmt::Display for PoolError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout(tt) => match tt {
                TimeoutType::Wait => write!(
                    f,
                    "A timeout occured while waiting for a slot to become available"
                ),
                TimeoutType::Create => write!(f, "A timeout occured while creating a new object"),
                TimeoutType::Recycle => write!(f, "A timeout occured while recycling an object"),
            },
            Self::Backend(e) => write!(f, "An error occured while creating a new object: {}", e),
            Self::Closed => write!(f, "The pool has been closed."),
            Self::NoRuntimeSpecified => write!(f, "No runtime specified."),
            Self::PostCreateHook(msg) => writeln!(f, "post_create hook failed: {}", msg),
            Self::PostRecycleHook(msg) => writeln!(f, "post_recycle hook failed: {}", msg),
        }
    }
}

impl<E> std::error::Error for PoolError<E> where E: std::error::Error {}
