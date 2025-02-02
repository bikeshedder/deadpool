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
async fn test_resize() {
    let mgr = Manager {};
    let pool = Pool::builder(mgr).max_size(2).build().unwrap();
    let obj1 = pool.get().await.unwrap();
    let obj2 = pool.get().await.unwrap();
    assert!(pool.status().size == 2);
    assert!(pool.status().max_size == 2);
    pool.resize(0);
    assert!(pool.status().size == 2);
    assert!(pool.status().max_size == 0);
    pool.resize(1);
    assert!(pool.status().size == 2);
    assert!(pool.status().max_size == 2);
    drop(obj1);
    drop(obj2);
}
