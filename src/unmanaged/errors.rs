use std::fmt;

/// Error structure for `Pool::get`
#[derive(Debug)]
pub enum PoolError {
    /// A timeout happened
    Timeout,
    /// The pool has been closed
    Closed,
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => write!(
                f,
                "A timeout occured while waiting for an object to become available"
            ),
            Self::Closed => write!(f, "The pool has been closed."),
        }
    }
}

impl std::error::Error for PoolError {}
