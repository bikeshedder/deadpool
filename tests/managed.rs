#[cfg(feature = "managed")]
mod tests {

    use std::time::Duration;

    use async_trait::async_trait;
    use tokio::time::sleep;

    use deadpool::managed::{Object, Pool, RecycleResult};

    struct Manager {}

    #[async_trait]
    impl deadpool::managed::Manager<usize, ()> for Manager {
        async fn create(&self) -> Result<usize, ()> {
            Ok(0)
        }
        async fn recycle(&self, _conn: &mut usize) -> RecycleResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_managed_basic() {
        let mgr = Manager {};
        let pool = Pool::new(mgr, 16);

        let status = pool.status();
        assert_eq!(status.size, 0);
        assert_eq!(status.available, 0);

        let obj0 = pool.get().await.unwrap();
        let status = pool.status();
        assert_eq!(status.size, 1);
        assert_eq!(status.available, 0);

        let obj1 = pool.get().await.unwrap();
        let status = pool.status();
        assert_eq!(status.size, 2);
        assert_eq!(status.available, 0);

        let obj2 = pool.get().await.unwrap();
        let status = pool.status();
        assert_eq!(status.size, 3);
        assert_eq!(status.available, 0);

        drop(obj0);
        let status = pool.status();
        assert_eq!(status.size, 3);
        assert_eq!(status.available, 1);

        drop(obj1);
        let status = pool.status();
        assert_eq!(status.size, 3);
        assert_eq!(status.available, 2);

        drop(obj2);
        let status = pool.status();
        assert_eq!(status.size, 3);
        assert_eq!(status.available, 3);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_managed_concurrent() {
        let mgr = Manager {};
        let pool = Pool::new(mgr, 3);

        // Spawn tasks
        let futures = (0..100)
            .map(|_| {
                let pool = pool.clone();
                tokio::spawn(async move {
                    let mut obj = pool.get().await.unwrap();
                    *obj += 1;
                    sleep(Duration::from_millis(1)).await;
                })
            })
            .collect::<Vec<_>>();

        // Await tasks to finish
        for future in futures {
            future.await.unwrap();
        }

        // Verify
        let status = pool.status();
        assert_eq!(status.size, 3);
        assert_eq!(status.available, 3);

        let values = [
            pool.get().await.unwrap(),
            pool.get().await.unwrap(),
            pool.get().await.unwrap(),
        ];

        assert_eq!(values.iter().map(|obj| **obj).sum::<usize>(), 100);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_managed_object_take() {
        let mgr = Manager {};
        let pool = Pool::new(mgr, 2);
        let obj0 = pool.get().await.unwrap();
        let obj1 = pool.get().await.unwrap();

        let status = pool.status();
        assert_eq!(status.size, 2);
        assert_eq!(status.available, 0);

        Object::take(obj0);
        let status = pool.status();
        assert_eq!(status.size, 1);
        assert_eq!(status.available, 0);

        Object::take(obj1);
        let status = pool.status();
        assert_eq!(status.size, 0);
        assert_eq!(status.available, 0);

        let obj0 = pool.get().await.unwrap();
        let obj1 = pool.get().await.unwrap();
        let status = pool.status();
        assert_eq!(status.size, 2);
        assert_eq!(status.available, 0);

        drop(obj0);
        drop(obj1);
        let status = pool.status();
        assert_eq!(status.size, 2);
        assert_eq!(status.available, 2);
    }
}
