#[cfg(feature = "managed")]
mod tests {

    use deadpool::managed::{Manager, ObjectCustomizer, Pool, RecycleResult, WrappedManager};

    struct Computer {}

    #[async_trait::async_trait]
    impl Manager for Computer {
        type Type = usize;
        type Error = ();
        async fn create(&self) -> Result<Self::Type, Self::Error> {
            Ok(42)
        }
        async fn recycle(&self, _: &mut Self::Type) -> RecycleResult<Self::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_nonblocking() {
        let manager = ObjectCustomizer::NonBlocking(|mut n: usize| {
            n += 1;
            n
        })
        .wrap_manager(Computer {});
        let pool = Pool::<WrappedManager<Computer, usize>>::new(manager, 1);
        assert!(*pool.get().await.unwrap() == 43);
    }

    #[tokio::test]
    async fn test_async() {
        let manager = ObjectCustomizer::Async(move |mut n| {
            Box::pin(async move {
                n += 1;
                n
            })
        })
        .wrap_manager(Computer {});
        let pool = Pool::<WrappedManager<Computer, usize>>::new(manager, 1);
        assert!(*pool.get().await.unwrap() == 43);
    }
}
