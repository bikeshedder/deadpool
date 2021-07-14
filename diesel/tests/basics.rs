#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use deadpool::managed::Pool;
    use deadpool_diesel::*;

    // These type aliases are repeated here as there is no
    // way to enable a crate local feature for the test profile:
    // https://github.com/rust-lang/cargo/issues/2911
    type SqliteConnection = Connection<diesel::SqliteConnection>;
    type SqliteManager = Manager<diesel::SqliteConnection>;
    type SqlitePool = Pool<SqliteManager, SqliteConnection>;

    fn create_pool(max_size: usize) -> SqlitePool {
        let manager = SqliteManager::new(":memory:");
        let pool = SqlitePool::new(manager, max_size);
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

        pool.get().await.unwrap();
    }

    #[tokio::test]
    async fn pooled_connection_impls_connection() {
        use diesel::prelude::*;
        use diesel::select;
        use diesel::sql_types::Text;

        let pool = create_pool(1);
        let mut conn = pool.get().await.unwrap();

        let query = select("foo".into_sql::<Text>());
        assert_eq!("foo", query.get_result::<String>(&mut conn).unwrap());
    }
}
