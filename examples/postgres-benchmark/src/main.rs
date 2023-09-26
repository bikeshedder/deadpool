use std::time::{Duration, Instant};

use deadpool_postgres::Runtime;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

const WORKERS: usize = 16;
const ITERATIONS: usize = 1000;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    #[serde(default)]
    pg: deadpool_postgres::Config,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()
            .unwrap()
            .try_deserialize()
    }
}

async fn without_pool(config: &Config) -> Duration {
    let pg_config = config.pg.get_pg_config().unwrap();
    let now = Instant::now();
    let (tx, mut rx) = mpsc::channel::<usize>(16);
    for i in 0..WORKERS {
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
    let pool = config
        .pg
        .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
        .unwrap();
    let now = Instant::now();
    let (tx, mut rx) = mpsc::channel::<usize>(16);
    for i in 0..WORKERS {
        let pool = pool.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            for _ in 0..ITERATIONS {
                let client = pool.get().await.unwrap();
                let stmt = client.prepare_cached("SELECT 1 + 2").await.unwrap();
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
