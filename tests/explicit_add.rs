use async_trait::async_trait;

use deadpool::{Pool, RecycleError, RecycleResult};

struct Manager;

#[async_trait]
impl deadpool::Manager<(), ()> for Manager {
    async fn create(&self) -> Result<(), ()> {
        Err(())
    }

    async fn recycle(&self, _: &mut ()) -> RecycleResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn test_explicit_add() {
    let pool = Pool::new(Manager, 1);

    assert!(pool.add(()));
    assert!(pool.get().await.is_ok());
}

#[tokio::test]
async fn test_explicit_add_fail() {
    let pool = Pool::new(Manager, 1);

    assert!(pool.add(()));
    assert!(pool.get().await.is_ok());

    assert!(!pool.add(()));
}

#[tokio::test]
async fn test_explicit_add_while_borrowed() {
    let pool = Pool::new(Manager, 1);

    assert!(pool.add(()));
    let value = pool.get().await.unwrap();

    assert!(!pool.add(()));
}
