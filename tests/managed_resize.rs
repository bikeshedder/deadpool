#![cfg(feature = "managed")]

use std::convert::Infallible;

use deadpool::managed::{self, Metrics, Object, RecycleResult};

type Pool = managed::Pool<Manager, Object<Manager>>;

struct Manager {}

impl managed::Manager for Manager {
    type Type = ();
    type Error = Infallible;

    async fn create(&self) -> Result<(), Infallible> {
        Ok(())
    }

    async fn recycle(&self, _conn: &mut (), _: &Metrics) -> RecycleResult<Infallible> {
        Ok(())
    }
}

#[tokio::test]
async fn test_grow_reuse_existing() {
    // Shrink doesn't discard objects currently borrowed from the pool but
    // keeps track of them so that repeatedly growing and shrinking will
    // not cause excessive object creation. This logic used to contain a bug
    // causing an overflow.
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(2).build().unwrap();
    let obj1 = pool.get().await.unwrap();
    let obj2 = pool.get().await.unwrap();
    assert!(pool.status().size == 2);
    assert!(pool.status().max_size == 2);
    pool.resize(0);
    // At this point the two objects are still tracked
    assert!(pool.status().size == 2);
    assert!(pool.status().max_size == 0);
    pool.resize(1);
    // Only one of the objects should be returned to the pool
    assert!(pool.status().size == 2);
    assert!(pool.status().max_size == 1);
    drop(obj1);
    // The first drop brings the size to 1.
    assert!(pool.status().size == 1);
    assert!(pool.status().max_size == 1);
    drop(obj2);
    assert!(pool.status().size == 1);
    assert!(pool.status().max_size == 1);
}
