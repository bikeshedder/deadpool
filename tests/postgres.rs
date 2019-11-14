#[tokio::main]
#[test]
#[cfg(feature = "postgres")]
async fn test_postgres() {
    use std::env;
    use deadpool::Pool;
    use deadpool::postgres::Manager as PgManager;
    let mut cfg = tokio_postgres::Config::new();
    cfg.host("/var/run/postgresql");
    cfg.user(env::var("USER").unwrap().as_str());
    cfg.dbname("deadpool");
    let mgr = PgManager::new(cfg, tokio_postgres::NoTls);
    let pool = Pool::new(mgr, 16);
    let mut client = pool.get().await.unwrap();
    let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();
    let value: i32 = rows[0].get(0);
    assert_eq!(value, 3);
}
