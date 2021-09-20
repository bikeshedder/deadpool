#![cfg(feature = "managed")]

use async_trait::async_trait;

use deadpool::{
    managed::{sync::SyncWrapper, Manager, Pool, RecycleResult},
    Runtime,
};

struct Computer {
    pub answer: usize,
}

struct ComputerManager {}

#[async_trait]
impl Manager for ComputerManager {
    type Type = SyncWrapper<Computer, ()>;
    type Error = ();

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        SyncWrapper::new(Runtime::Tokio1, || Ok(Computer { answer: 42 })).await
    }

    async fn recycle(&self, _: &mut Self::Type) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

#[tokio::test]
async fn post_recycle() {
    let manager = ComputerManager {};
    let pool = Pool::<ComputerManager>::builder(manager)
        .max_size(1)
        .build()
        .unwrap();
    let obj = pool.get().await.unwrap();
    assert_eq!(
        obj.interact(|computer| Ok(computer.answer)).await.unwrap(),
        42
    );
    let guard = obj.lock().unwrap();
    assert_eq!(guard.answer, 42);
}
