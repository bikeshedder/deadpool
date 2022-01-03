#![cfg(feature = "managed")]

use std::{convert::Infallible, time::Duration};

use async_trait::async_trait;
use tokio::time;

use deadpool::managed::{self, Object, PoolError, RecycleResult};

type Pool = managed::Pool<Manager>;

struct Manager {}

#[async_trait]
impl managed::Manager for Manager {
    type Type = usize;
    type Error = Infallible;

    async fn create(&self) -> Result<usize, Infallible> {
        Ok(0)
    }

    async fn recycle(&self, _conn: &mut usize) -> RecycleResult<Infallible> {
        Ok(())
    }
}

#[tokio::test]
async fn basic() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(16).build().unwrap();

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

#[tokio::test]
async fn closing() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(1).build().unwrap();

    // fetch the only object from the pool
    let obj = pool.get().await;
    let join_handle = {
        let pool = pool.clone();
        tokio::spawn(async move { pool.get().await })
    };

    tokio::task::yield_now().await;
    assert_eq!(pool.status().available, -1);

    pool.close();
    tokio::task::yield_now().await;
    assert_eq!(pool.status().available, 0);

    assert!(matches!(join_handle.await.unwrap(), Err(PoolError::Closed)));
    assert!(matches!(pool.get().await, Err(PoolError::Closed)));
    assert!(matches!(pool.try_get().await, Err(PoolError::Closed)));

    drop(obj);
    tokio::task::yield_now().await;
    assert_eq!(pool.status().available, 0);
}

#[tokio::test]
async fn close_resize() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(1).build().unwrap();
    pool.close();
    pool.resize(16);
    assert_eq!(pool.status().size, 0);
    assert_eq!(pool.status().max_size, 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(3).build().unwrap();

    // Spawn tasks
    let futures = (0..100)
        .map(|_| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let mut obj = pool.get().await.unwrap();
                *obj += 1;
                time::sleep(Duration::from_millis(1)).await;
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
async fn object_take() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(2).build().unwrap();
    let obj0 = pool.get().await.unwrap();
    let obj1 = pool.get().await.unwrap();

    let status = pool.status();
    assert_eq!(status.size, 2);
    assert_eq!(status.available, 0);

    let _ = Object::take(obj0);
    let status = pool.status();
    assert_eq!(status.size, 1);
    assert_eq!(status.available, 0);

    let _ = Object::take(obj1);
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

#[tokio::test]
async fn resize_pool_shrink() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(2).build().unwrap();
    let obj0 = pool.get().await.unwrap();
    let obj1 = pool.get().await.unwrap();
    pool.resize(1);
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 2);
    drop(obj1);
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 1);
    drop(obj0);
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 1);
}

#[tokio::test]
async fn resize_pool_grow() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(1).build().unwrap();
    let obj0 = pool.get().await.unwrap();
    pool.resize(2);
    assert_eq!(pool.status().max_size, 2);
    assert_eq!(pool.status().size, 1);
    let obj1 = pool.get().await.unwrap();
    assert_eq!(pool.status().max_size, 2);
    assert_eq!(pool.status().size, 2);
    drop(obj1);
    assert_eq!(pool.status().max_size, 2);
    assert_eq!(pool.status().size, 2);
    drop(obj0);
    assert_eq!(pool.status().max_size, 2);
    assert_eq!(pool.status().size, 2);
}

#[tokio::test]
async fn resize_pool_shrink_grow() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(1).build().unwrap();
    let obj0 = pool.get().await.unwrap();
    pool.resize(2);
    pool.resize(0);
    pool.resize(5);
    assert_eq!(pool.status().max_size, 5);
    assert_eq!(pool.status().size, 1);
    drop(obj0);
    assert_eq!(pool.status().max_size, 5);
    assert_eq!(pool.status().size, 1);
}

#[tokio::test]
async fn resize_pool_grow_concurrent() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(0).build().unwrap();
    let join_handle = {
        let pool = pool.clone();
        tokio::spawn(async move { pool.get().await })
    };
    tokio::task::yield_now().await;
    assert_eq!(pool.status().max_size, 0);
    assert_eq!(pool.status().size, 0);
    assert_eq!(pool.status().available, -1);
    pool.resize(1);
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 0);
    assert_eq!(pool.status().available, -1);
    tokio::task::yield_now().await;
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(pool.status().available, 0);
    let obj0 = join_handle.await.unwrap().unwrap();
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(pool.status().available, 0);
    drop(obj0);
    assert_eq!(pool.status().max_size, 1);
    assert_eq!(pool.status().size, 1);
    assert_eq!(pool.status().available, 1);
}
