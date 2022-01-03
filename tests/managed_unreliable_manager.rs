#![cfg(feature = "managed")]

use std::time::Duration;

use async_trait::async_trait;
use tokio::time;

use deadpool::managed::{self, RecycleError, RecycleResult};

type Pool = managed::Pool<Manager>;

struct Manager {
    create_fail: bool,
    recycle_fail: bool,
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = ();
    type Error = ();

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

#[tokio::test]
async fn create() {
    let manager = Manager {
        create_fail: true,
        recycle_fail: false,
    };

    let pool = Pool::builder(manager).max_size(16).build().unwrap();
    {
        assert!(!pool.get().await.is_ok());
    }

    let status = pool.status();
    assert_eq!(status.available, 0);
    assert_eq!(status.size, 0);
    {
        assert!(!time::timeout(Duration::from_millis(10), pool.get())
            .await
            .unwrap()
            .is_ok(),);
    }
    assert_eq!(status.available, 0);
    assert_eq!(status.size, 0);
}

#[tokio::test]
async fn recycle() {
    let manager = Manager {
        create_fail: false,
        recycle_fail: true,
    };

    let pool = Pool::builder(manager).max_size(16).build().unwrap();
    {
        let _a = pool.get().await.unwrap();
        let _b = pool.get().await.unwrap();
    }

    let status = pool.status();
    assert_eq!(status.available, 2);
    assert_eq!(status.size, 2);
    {
        let _a = pool.get().await.unwrap();
        // All connections fail to recycle. Thus reducing the
        // available counter to 0.
        let status = pool.status();
        assert_eq!(status.available, 0);
        assert_eq!(status.size, 1);
    }
    let status = pool.status();
    assert_eq!(status.available, 1);
    assert_eq!(status.size, 1);
}
