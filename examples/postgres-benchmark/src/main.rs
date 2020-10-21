use std::time::{Duration, Instant};

use dotenv::dotenv;
use serde::Deserialize;

const WORKERS: usize = 16;
const ITERATIONS: usize = 1000;

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    pg: deadpool_postgres::Config,
}

impl Config {
    pub fn from_env() -> Result<Self, ::config::ConfigError> {
        let mut cfg = ::config::Config::new();
        cfg.merge(::config::Environment::new().separator("__"))?;
        cfg.try_into()
    }
}

async fn without_pool(config: &Config) -> Duration {
    let pg_config = config.pg.get_pg_config().unwrap();
    let now = Instant::now();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<usize>(16);
    for i in 0usize..WORKERS {
        let tx = tx.clone();
        let pg_config = pg_config.clone();
        tokio::spawn(async move {
            for _ in 0..ITERATIONS {
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
    for _ in 0..WORKERS {
        rx.recv().await.unwrap();
    }
    now.elapsed()
}

async fn with_deadpool(config: &Config) -> Duration {
    let pool = config.pg.create_pool(tokio_postgres::NoTls).unwrap();
    let now = Instant::now();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<usize>(16);
    for i in 0..WORKERS {
        let pool = pool.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            for _ in 0..ITERATIONS {
                let client = pool.get().await.unwrap();
                let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
                let rows = client.query(&stmt, &[]).await.unwrap();
                let value: i32 = rows[0].get(0);
                assert_eq!(value, 3);
            }
            tx.send(i).await.unwrap();
        });
    }
    for _ in 0usize..WORKERS {
        rx.recv().await.unwrap();
    }
    now.elapsed()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let cfg = Config::from_env()?;
    let d1 = without_pool(&cfg).await;
    println!("Without pool: {}ms", d1.as_millis());
    let d2 = with_deadpool(&cfg).await;
    println!("With pool: {}ms", d2.as_millis());
    println!("Speedup: {}%", 100 * d1.as_millis() / d2.as_millis());
    assert!(d1 > d2);
    Ok(())
}
