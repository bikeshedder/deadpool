use deadpool_postgres::{Manager, Pool};
use std::env;

fn create_pool() -> Pool {
    let mut cfg = tokio_postgres::Config::new();
    cfg.host("/var/run/postgresql");
    cfg.user(env::var("USER").unwrap().as_str());
    cfg.dbname("deadpool");
    let mgr = Manager::new(cfg, tokio_postgres::NoTls);
    Pool::new(mgr, 16)
}

#[tokio::main]
#[test]
async fn test_basic() {
    let pool = create_pool();
    let mut client = pool.get().await.unwrap();
    let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();
    let value: i32 = rows[0].get(0);
    assert_eq!(value, 3);
}

#[tokio::main]
#[test]
async fn test_transaction_1() {
    let pool = create_pool();
    let mut client = pool.get().await.unwrap();
    {
        let mut txn = client.transaction().await.unwrap();
        let stmt = txn.prepare("SELECT 1 + 2").await.unwrap();
        let rows = txn.query(&stmt, &[]).await.unwrap();
        let value: i32 = rows[0].get(0);
        assert_eq!(value, 3);
    }
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
        assert_eq!(value, 3);
    }
}
