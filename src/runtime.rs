//! Runtime specific feature
use std::any::Any;
use std::future::Future;
use std::time::Duration;

#[derive(Clone, Debug)]
/// Enumeration for picking a runtime implementation
pub enum Runtime {
    /// Disable runtime specific features
    /// This disables timeouts and the possibility to spawn async tasks.
    None,
    /// tokio 1.0 runtime
    #[cfg(feature = "rt_tokio_1")]
    Tokio1,
    /// async-std 1.0 runtime
    #[cfg(feature = "rt_async-std_1")]
    AsyncStd1,
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime::None
    }
}

pub enum TimeoutError {
    Timeout,
    NoRuntime,
}

pub enum SpawnBlockingError {
    NoRuntime,
    Panic(Box<dyn Any + Send + 'static>),
}

impl Runtime {
    /// Require a Future to complete before the specified duration has elapsed.
    ///
    /// If the future completes before the duration has elapsed, then the
    /// completed value is returned. Otherwise, an error is returned and
    /// the future is canceled.
    #[allow(unused_variables)]
    pub async fn timeout<F>(&self, duration: Duration, future: F) -> Result<F::Output, TimeoutError>
    where
        F: Future,
    {
        match self {
            Self::None => Err(TimeoutError::NoRuntime),
            #[cfg(feature = "rt_tokio_1")]
            Self::Tokio1 => tokio::time::timeout(duration, future)
                .await
                .map_err(|_| TimeoutError::Timeout),
            #[cfg(feature = "rt_async-std_1")]
            Self::AsyncStd1 => async_std::future::timeout(duration, future)
                .await
                .map_err(|_| TimeoutError::Timeout),
        }
    }

    /// Run the closure on a thread where blocking is acceptable.
    #[allow(unused_variables)]
    pub async fn spawn_blocking<F, R>(&self, f: F) -> Result<R, SpawnBlockingError>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        match self {
            Self::None => Err(SpawnBlockingError::NoRuntime),
            #[cfg(feature = "rt_tokio_1")]
            Self::Tokio1 => tokio::task::spawn_blocking(f)
                .await
                .map_err(|e| SpawnBlockingError::Panic(e.into_panic())),
            #[cfg(feature = "rt_async-std_1")]
            Self::AsyncStd1 => Ok(async_std::task::spawn_blocking(f).await),
        }
    }
}
