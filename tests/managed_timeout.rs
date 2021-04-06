#[cfg(feature = "managed")]
mod tests {

    use std::{convert::Infallible, future::Future, time::Duration};

    use async_trait::async_trait;

    use deadpool::managed::{PoolConfig, PoolError, RecycleResult, Timeouts};
    use deadpool::Runtime;
    type Pool = deadpool::managed::Pool<Manager>;

    struct Manager {}

    struct Never();
    impl Future for Never {
        type Output = ();
        fn poll(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            std::task::Poll::Pending
        }
    }

    #[async_trait]
    impl deadpool::managed::Manager for Manager {
        type Type = usize;
        type Error = Infallible;
        async fn create(&self) -> Result<usize, Infallible> {
            Never().await;
            unreachable!();
        }
        async fn recycle(&self, _conn: &mut usize) -> RecycleResult<Infallible> {
            Never().await;
            unreachable!();
        }
    }

    async fn _test_managed_timeout(runtime: Runtime) {
        let mgr = Manager {};
        let cfg = PoolConfig {
            max_size: 16,
            timeouts: Timeouts {
                create: Some(Duration::from_millis(0)),
                wait: Some(Duration::from_millis(0)),
                recycle: Some(Duration::from_millis(0)),
            },
            runtime,
        };
        let pool = Pool::from_config(mgr, cfg);
        assert!(matches!(pool.get().await, Err(PoolError::Timeout(_))));
    }

    #[cfg(feature = "rt_tokio_1")]
    #[tokio::test]
    async fn test_rt_tokio_1() {
        _test_managed_timeout(Runtime::Tokio1).await;
    }

    #[cfg(feature = "rt_async-std_1")]
    #[async_std::test]
    async fn test_rt_async_std_1() {
        _test_managed_timeout(Runtime::AsyncStd1).await;
    }
}
