#![cfg(feature = "managed")]

use async_trait::async_trait;

use deadpool::managed::{
    hooks::{HookError, PostCreate, PostRecycle},
    Manager, Pool, RecycleResult, Statistics,
};

struct Computer {}

#[async_trait]
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

struct IncrementHook {}

#[async_trait]
impl PostCreate<Computer> for IncrementHook {
    async fn post_create(&self, obj: &mut usize) -> Result<(), HookError<()>> {
        *obj += 1;
        Ok(())
    }
}

#[async_trait]
impl PostRecycle<Computer> for IncrementHook {
    async fn post_recycle(&self, obj: &mut usize, _: &Statistics) -> Result<(), HookError<()>> {
        *obj += 1;
        Ok(())
    }
}

#[tokio::test]
async fn post_create() {
    let manager = Computer {};
    let pool = Pool::<Computer>::builder(manager)
        .max_size(1)
        .post_create(IncrementHook {})
        .build()
        .unwrap();
    assert!(*pool.get().await.unwrap() == 43);
}

#[tokio::test]
async fn post_recycle() {
    let manager = Computer {};
    let pool = Pool::<Computer>::builder(manager)
        .max_size(1)
        .post_recycle(IncrementHook {})
        .build()
        .unwrap();
    assert!(*pool.get().await.unwrap() == 42);
    assert!(*pool.get().await.unwrap() == 43);
    assert!(*pool.get().await.unwrap() == 44);
    assert!(*pool.get().await.unwrap() == 45);
}
