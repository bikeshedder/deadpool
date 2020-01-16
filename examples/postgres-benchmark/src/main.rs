use dotenv::dotenv;
use deadpool_postgres::Config;
use std::time::Instant;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let workers = 16;
    let iterations = 1000;
    let cfg = Config::from_env("PG").unwrap();
    // without pool
    let d1 = {
        let pg_config = cfg.get_pg_config().unwrap();
        let now = Instant::now();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<usize>(16);
        for i in 0usize..workers {
            let mut tx = tx.clone();
            let pg_config = pg_config.clone();
            tokio::spawn(async move {
                for _ in 0..iterations {
                    let (client, connection) = pg_config.connect(tokio_postgres::NoTls).await.unwrap();
                    tokio::spawn(connection);
                    let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
                    let rows = client.query(&stmt, &[]).await.unwrap();
                    let value: i32 = rows[0].get(0);
                    assert_eq!(value, 3);
                }
                tx.send(i).await.unwrap();
            });
        }
        for _ in 0usize..workers {
            rx.recv().await.unwrap();
        }
        now.elapsed()
    };
    println!("Without pool: {}ms", d1.as_millis());
    // with pool (16 clients in parallel)
    let pool = cfg.create_pool(tokio_postgres::NoTls).unwrap();
    let now = Instant::now();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<usize>(16);
    for i in 0..workers {
        let pool = pool.clone();
        let mut tx = tx.clone();
        tokio::spawn(async move {
            for _ in 0..iterations {
                let client = pool.get().await.unwrap();
                let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
                let rows = client.query(&stmt, &[]).await.unwrap();
                let value: i32 = rows[0].get(0);
                assert_eq!(value, 3);
            }
            tx.send(i).await.unwrap();
        });
    }
    for _ in 0usize..workers {
        rx.recv().await.unwrap();
    }
    let d2 = now.elapsed();
    println!("With pool: {}ms", d2.as_millis());
    println!("Speedup: {}%", 100 * d1.as_millis() / d2.as_millis());
    assert!(d1 > d2);
}
