//! Runtime-specific features.

use std::{any::Any, fmt, future::Future, time::Duration};

/// Enumeration for picking a runtime implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Runtime {
    #[cfg(feature = "rt_tokio_1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt_tokio_1")))]
    /// [`tokio` 1.0](tokio) runtime.
    Tokio1,

    #[cfg(feature = "rt_async-std_1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt_async-std_1")))]
    /// [`async-std` 1.0](async_std) runtime.
    AsyncStd1,
}

impl Runtime {
    /// Requires a [`Future`] to complete before the specified `duration` has
    /// elapsed.
    ///
    /// If the `future` completes before the `duration` has elapsed, then the
    /// completed value is returned. Otherwise, an error is returned and
    /// the `future` is canceled.
    #[allow(unused_variables)]
    pub async fn timeout<F>(&self, duration: Duration, future: F) -> Option<F::Output>
    where
        F: Future,
    {
        match self {
            #[cfg(feature = "rt_tokio_1")]
            Self::Tokio1 => tokio::time::timeout(duration, future).await.ok(),
            #[cfg(feature = "rt_async-std_1")]
            Self::AsyncStd1 => async_std::future::timeout(duration, future).await.ok(),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }

    /// Runs the given closure on a thread where blocking is acceptable.
    ///
    /// # Errors
    ///
    /// See [`SpawnBlockingError`] for details.
    #[allow(unused_variables)]
    pub async fn spawn_blocking<F, R>(&self, f: F) -> Result<R, SpawnBlockingError>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        match self {
            #[cfg(feature = "rt_tokio_1")]
            Self::Tokio1 => tokio::task::spawn_blocking(f)
                .await
                .map_err(|e| SpawnBlockingError::Panic(e.into_panic())),
            #[cfg(feature = "rt_async-std_1")]
            Self::AsyncStd1 => Ok(async_std::task::spawn_blocking(f).await),
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }

    /// Runs the given closure on a thread where blocking is acceptable.
    ///
    /// It works similar to [`Runtime::spawn_blocking()`] but doesn't return a
    /// [`Future`] and is meant to be used for background tasks.
    ///
    /// # Errors
    ///
    /// See [`SpawnBlockingError`] for details.
    #[allow(unused_variables)]
    pub fn spawn_blocking_background<F>(&self, f: F) -> Result<(), SpawnBlockingError>
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            #[cfg(feature = "rt_tokio_1")]
            Self::Tokio1 => {
                drop(tokio::task::spawn_blocking(f));
                Ok(())
            }
            #[cfg(feature = "rt_async-std_1")]
            Self::AsyncStd1 => {
                drop(async_std::task::spawn_blocking(f));
                Ok(())
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}

/// Error of spawning a task on a thread where blocking is acceptable.
#[derive(Debug)]
pub enum SpawnBlockingError {
    /// Spawned task has panicked.
    Panic(Box<dyn Any + Send + 'static>),
}

impl fmt::Display for SpawnBlockingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panic(p) => write!(f, "SpawnBlockingError: Panic: {:?}", p),
        }
    }
}

impl std::error::Error for SpawnBlockingError {}
