#[tokio::main]
#[cfg(feature = "postgres")]
async fn main() {
    use std::env;
    use std::time::{Instant};
    use deadpool::Pool;
    use deadpool::postgres::Manager as PgManager;
    let mut cfg = tokio_postgres::Config::new();
    cfg.host("/var/run/postgresql");
    cfg.user(env::var("USER").unwrap().as_str());
    cfg.dbname("deadpool");
    let mgr = PgManager::new(cfg, tokio_postgres::NoTls);
    let pool = Pool::new(mgr, 16);
    // without pool (just using one client of it)
    let now = Instant::now();
    {
        let mut client = pool.get().await.unwrap();
        for _ in 0usize..16000usize {
            let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
            let rows = client.query(&stmt, &[]).await.unwrap();
            let value: i32 = rows[0].get(0);
            assert_eq!(value, 3);
        }
    }
    let d1 = now.elapsed();
    println!("Without pool: {}ms", d1.as_millis());
    // with pool (16 clients in parallel)
    let now = Instant::now();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<usize>(16);
    let count = 16;
    for i in 0usize..count {
        let pool = pool.clone();
        let mut tx = tx.clone();
        tokio::spawn(async move {
            for _ in 0usize..1000usize {
                let mut client = pool.get().await.unwrap();
                let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
                let rows = client.query(&stmt, &[]).await.unwrap();
                let value: i32 = rows[0].get(0);
                assert_eq!(value, 3);
            }
            tx.send(i).await.unwrap();
        });
    }
    for _ in 0usize..count {
        rx.recv().await.unwrap();
    }
    let d2 = now.elapsed();
    println!("With pool: {}ms", d2.as_millis());
    println!("Speedup: {}%", 100 * d1.as_millis() / d2.as_millis());
    assert!(d1 > d2);
}
