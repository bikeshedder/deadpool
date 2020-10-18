#[cfg(feature = "managed")]
mod tests {

    use std::sync::Arc;
    use std::time::Duration;

    use async_trait::async_trait;
    use tokio::sync::mpsc::{channel, Receiver, Sender};
    use tokio::sync::Mutex;
    use tokio::task::yield_now;
    use tokio::time::timeout;

    use deadpool::managed::{Pool, RecycleError, RecycleResult};

    struct Manager {
        create_rx: Arc<Mutex<Receiver<Result<(), ()>>>>,
        recycle_rx: Arc<Mutex<Receiver<Result<(), ()>>>>,
        remote_control: RemoteControl,
    }

    #[derive(Clone)]
    struct RemoteControl {
        create_tx: Sender<Result<(), ()>>,
        recycle_tx: Sender<Result<(), ()>>,
    }

    impl RemoteControl {
        pub fn create_ok(&mut self) {
            self.create_tx.try_send(Ok(())).unwrap();
        }
        pub fn create_err(&mut self) {
            self.create_tx.try_send(Err(())).unwrap();
        }
        /*
        pub fn recycle_ok(&mut self) {
            self.recycle_tx.try_send(Ok(())).unwrap();
        }
        pub fn recycle_err(&mut self) {
            self.recycle_tx.try_send(Err(())).unwrap();
        }
        */
    }

    impl Manager {
        pub fn new() -> Self {
            let (create_tx, create_rx) = channel(16);
            let (recycle_tx, recycle_rx) = channel(16);
            Self {
                create_rx: Arc::new(Mutex::new(create_rx)),
                recycle_rx: Arc::new(Mutex::new(recycle_rx)),
                remote_control: RemoteControl {
                    create_tx,
                    recycle_tx,
                },
            }
        }
    }

    #[async_trait]
    impl deadpool::managed::Manager<(), ()> for Manager {
        async fn create(&self) -> Result<(), ()> {
            self.create_rx.lock().await.recv().await.unwrap()
        }
        async fn recycle(&self, _conn: &mut ()) -> RecycleResult<()> {
            match self.recycle_rx.lock().await.recv().await.unwrap() {
                Ok(()) => Ok(()),
                Err(e) => Err(RecycleError::Backend(e)),
            }
        }
    }

    // When the pool is drained, all connections fail to create and the
    #[tokio::main(flavor = "current_thread")]
    #[test]
    async fn test_pool_drained() {
        let manager = Manager::new();
        let mut rc = manager.remote_control.clone();
        let pool = Pool::new(manager, 1);
        let pool_clone = pool.clone();
        // let first task grab the only connection
        let get_1 = tokio::spawn(async move {
            pool_clone.get().await.unwrap();
        });
        yield_now().await;
        assert_eq!(pool.status().size, 1);
        assert_eq!(pool.status().available, 0);
        // let second task wait for the connection
        let pool_clone = pool.clone();
        let get_2 = tokio::spawn(async move {
            pool_clone.get().await.unwrap();
        });
        yield_now().await;
        assert_eq!(pool.status().size, 1);
        assert_eq!(pool.status().available, -1);
        // first task receives an error
        rc.create_err();
        assert!(get_1.await.is_err());
        // the second task should now be able to create an object
        rc.create_ok();
        let result = timeout(Duration::from_millis(10), get_2).await;
        assert!(result.is_ok(), "get_2 should not time out");
        assert!(result.unwrap().is_ok(), "get_2 should receive an object");
        assert_eq!(pool.status().size, 1);
        assert_eq!(pool.status().available, 1);
    }
}
