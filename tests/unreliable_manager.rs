use std::time::Duration;

use async_trait::async_trait;
use tokio::time::timeout;

use deadpool::{Pool, RecycleError, RecycleResult};

struct Manager {
    create_fail: bool,
    recycle_fail: bool,
}

#[async_trait]
impl deadpool::Manager<(), ()> for Manager {
    async fn create(&self) -> Result<(), ()> {
        if self.create_fail {
            Err(())
        } else {
            Ok(())
        }
    }
    async fn recycle(&self, _conn: &mut ()) -> RecycleResult<()> {
        if self.recycle_fail {
            Err(RecycleError::Backend(()))
        } else {
            Ok(())
        }
    }
}

#[tokio::main]
#[test]
async fn test_unreliable_create() {
    let manager = Manager { create_fail: true, recycle_fail: false };
    let pool = Pool::new(manager, 16);
    {
        assert_eq!(pool.get().await.is_ok(), false);
    }
    let status = pool.status();
    assert_eq!(status.available, 0);
    assert_eq!(status.size, 0);
    {
        assert_eq!(timeout(Duration::from_millis(10), pool.get()).await.unwrap().is_ok(), false);
    }
    assert_eq!(status.available, 0);
    assert_eq!(status.size, 0);
}

#[tokio::main]
#[test]
async fn test_unreliable_recycle() {
    let manager = Manager { create_fail: false, recycle_fail: true };
    let pool = Pool::new(manager, 16);
    {
        assert_eq!(pool.get().await.is_ok(), true);
    }
    let status = pool.status();
    assert_eq!(status.available, 1);
    assert_eq!(status.size, 1);
    {
        assert_eq!(pool.get().await.is_ok(), true);
    }
    assert_eq!(status.available, 1);
    assert_eq!(status.size, 1);
}
