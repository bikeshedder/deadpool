use dotenv::dotenv;
use deadpool_postgres::Config;
use std::time::Instant;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cfg = Config::from_env("PG").unwrap();
    let pool = cfg.create_pool(tokio_postgres::NoTls).unwrap();
    // without pool (just using one client of it)
    let now = Instant::now();
    {
        let client = pool.get().await.unwrap();
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
                let client = pool.get().await.unwrap();
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
