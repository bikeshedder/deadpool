#[cfg(feature = "unmanaged")]
mod tests {

    use std::time::Duration;

    use deadpool::unmanaged::{PoolConfig, PoolError};
    use deadpool::Runtime;

    type Pool = deadpool::unmanaged::Pool<()>;

    #[tokio::test]
    async fn test_timeout_no_runtime() {
        let pool = Pool::default();
        assert!(matches!(
            pool.timeout_get(Some(Duration::from_millis(1))).await,
            Err(PoolError::NoRuntimeSpecified)
        ));
        assert!(matches!(
            pool.timeout_get(Some(Duration::from_millis(0))).await,
            Err(PoolError::Timeout)
        ));
    }

    async fn _test_unmanaged_timeout_get(runtime: Runtime) {
        let cfg = PoolConfig {
            max_size: 16,
            timeout: None,
            runtime,
        };
        let pool = Pool::from_config(&cfg);
        assert!(matches!(
            pool.timeout_get(Some(Duration::from_millis(1))).await,
            Err(PoolError::Timeout)
        ));
    }

    async fn _test_unmanaged_timeout_config(runtime: Runtime) {
        let cfg = PoolConfig {
            max_size: 16,
            timeout: Some(Duration::from_millis(1)),
            runtime,
        };
        let pool = Pool::from_config(&cfg);
        assert!(matches!(pool.get().await, Err(PoolError::Timeout)));
    }

    #[cfg(feature = "rt_tokio_1")]
    #[tokio::test]
    async fn test_rt_tokio_1() {
        _test_unmanaged_timeout_get(Runtime::Tokio1).await;
        _test_unmanaged_timeout_config(Runtime::Tokio1).await;
    }

    #[cfg(feature = "rt_async-std_1")]
    #[async_std::test]
    async fn test_rt_async_std_1() {
        _test_unmanaged_timeout_get(Runtime::AsyncStd1).await;
        _test_unmanaged_timeout_config(Runtime::AsyncStd1).await;
    }
}
