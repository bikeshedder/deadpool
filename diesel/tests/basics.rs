#![cfg(feature = "sqlite")]

use tokio::sync::mpsc;

use deadpool_diesel::{Manager, Pool, Runtime};

type SqliteManager = Manager<diesel::SqliteConnection>;
type SqlitePool = Pool<SqliteManager>;

fn create_pool(max_size: usize) -> SqlitePool {
    let manager = SqliteManager::new(":memory:", Runtime::Tokio1);
    let pool = SqlitePool::builder(manager)
        .max_size(max_size)
        .build()
        .unwrap();
    pool
}

#[tokio::test]
async fn establish_basic_connection() {
    let pool = create_pool(2);

    let (s1, mut r1) = mpsc::channel(1);
    let (s2, mut r2) = mpsc::channel(1);

    let pool1 = pool.clone();
    let t1 = tokio::spawn(async move {
        let conn = pool1.get().await.unwrap();
        s1.send(()).await.unwrap();
        r2.recv().await.unwrap();
        drop(conn)
    });

    let pool2 = pool.clone();
    let t2 = tokio::spawn(async move {
        let conn = pool2.get().await.unwrap();
        s2.send(()).await.unwrap();
        r1.recv().await.unwrap();
        drop(conn)
    });

    t1.await.unwrap();
    t2.await.unwrap();

    drop(pool.get().await.unwrap());
}

#[tokio::test]
async fn pooled_connection_impls_connection() {
    use diesel::prelude::*;
    use diesel::select;
    use diesel::sql_types::Text;

    let pool = create_pool(1);
    let conn = pool.get().await.unwrap();
    let result = conn
        .interact(|conn| {
            let query = select("foo".into_sql::<Text>());
            query.get_result::<String>(conn).map_err(Into::into)
        })
        .await
        .unwrap();
    assert_eq!("foo", &result);
}
