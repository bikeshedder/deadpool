use std::env;
use std::path::Path;

use futures::future::join_all;
use tokio_postgres::types::Type;

use deadpool_postgres::{Manager, Pool};

fn pg_config_from_env() -> tokio_postgres::config::Config {
    let mut config = tokio_postgres::Config::new();
    if let Ok(host) = env::var("PG_HOST") {
        config.host(host.as_str());
    } else if Path::new("/run/postgresql").exists() {
        config.host("/run/postgresql");
    } else {
        config.host("/tmp");
    }
    if let Ok(port) = env::var("PG_PORT") {
        match u16::from_str_radix(port.as_str(), 10) {
            Ok(port) => {
                config.port(port);
            }
            Err(_) => {
                panic!(format!("Invalid port: {}", port));
            }
        }
    }
    if let Ok(user) = env::var("PG_USER") {
        config.user(user.as_str());
    } else if let Ok(user) = env::var("USER") {
        config.user(user.as_str());
    } else {
        panic!("PG_USER missing in environment; fallback to USER failed as well.");
    }
    if let Ok(password) = env::var("PG_PASSWORD") {
        config.password(password.as_str());
    }
    if let Ok(dbname) = env::var("PG_DBNAME") {
        config.dbname(dbname.as_str());
    } else {
        config.dbname("deadpool");
    }
    config
}

fn create_pool() -> Pool {
    let cfg = pg_config_from_env();
    let mgr = Manager::new(cfg, tokio_postgres::NoTls);
    Pool::new(mgr, 16)
}

#[tokio::main]
#[test]
async fn test_basic() {
    let pool = create_pool();
    let client = pool.get().await.unwrap();
    let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();
    let value: i32 = rows[0].get(0);
    assert_eq!(value, 3);
    assert_eq!(client.statement_cache.size(), 1);
}

#[tokio::main]
#[test]
async fn test_prepare_typed() {
    let pool = create_pool();
    let client = pool.get().await.unwrap();
    let stmt = client
        .prepare_typed("SELECT 1 + $1", &[Type::INT2])
        .await
        .unwrap();
    let rows = client.query(&stmt, &[&42i16]).await.unwrap();
    let value: i32 = rows[0].get(0);
    assert_eq!(value, 43i32);
}

#[tokio::main]
#[test]
async fn test_prepare_typed_error() {
    let pool = create_pool();
    let client = pool.get().await.unwrap();
    let stmt = client
        .prepare_typed("SELECT 1 + $1", &[Type::INT2])
        .await
        .unwrap();
    assert!(client.query(&stmt, &[&42i32]).await.is_err());
}

#[tokio::main]
#[test]
async fn test_transaction_1() {
    let pool = create_pool();
    let mut client = pool.get().await.unwrap();
    {
        let txn = client.transaction().await.unwrap();
        let stmt = txn.prepare("SELECT 1 + 2").await.unwrap();
        let rows = txn.query(&stmt, &[]).await.unwrap();
        let value: i32 = rows[0].get(0);
        txn.commit().await.unwrap();
        assert_eq!(value, 3);
    }
    assert_eq!(client.statement_cache.size(), 1);
}

#[tokio::main]
#[test]
async fn test_transaction_2() {
    let pool = create_pool();
    let mut client = pool.get().await.unwrap();
    let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
    {
        let txn = client.transaction().await.unwrap();
        let rows = txn.query(&stmt, &[]).await.unwrap();
        let value: i32 = rows[0].get(0);
        txn.commit().await.unwrap();
        assert_eq!(value, 3);
    }
    assert_eq!(client.statement_cache.size(), 1);
}

#[tokio::main]
#[test]
async fn test_transaction_pipeline() {
    let pool = create_pool();
    let mut client = pool.get().await.unwrap();
    let stmt = client.prepare("SELECT 1 + $1").await.unwrap();
    let txn = client.transaction().await.unwrap();
    let mut futures = vec![];
    for i in 0..100 {
        let stmt = stmt.clone();
        let txn = &txn;
        futures.push(async move {
            let rows = txn.query(&stmt, &[&i]).await.unwrap();
            let value: i32 = rows[0].get(0);
            value
        });
    }
    let results = join_all(futures).await;
    for i in 0..100 {
        assert_eq!(results[i], (i as i32) + 1);
    }
}
