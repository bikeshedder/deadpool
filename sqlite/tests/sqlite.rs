use deadpool_sqlite::Config;

#[tokio::test]
async fn test_basic() {
    let cfg = Config {
        path: String::from("db.sqlite3"),
        pool: None,
    };
    let pool = cfg.create_pool();
    let conn = pool.get().await.unwrap();
    let result: i64 = conn
        .interact(|conn| {
            let mut stmt = conn.prepare("SELECT 1").unwrap();
            let mut rows = stmt.query([]).unwrap();
            let row = rows.next().unwrap().unwrap();
            row.get(0)
        })
        .await
        .unwrap();
    assert_eq!(result, 1);
}
