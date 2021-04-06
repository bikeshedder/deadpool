//! Runtime specific feature
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

impl Runtime {
    /// Require a Future to complete before the specified duration has elapsed.
    ///
    // If the future completes before the duration has elapsed, then the
    /// completed value is returned. Otherwise, an error is returned and
    /// the future is canceled.
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
}
