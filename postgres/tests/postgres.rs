use std::collections::HashMap;
use std::env;
use std::time::Duration;

use futures::future::join_all;
use serde::Deserialize;
use tokio_postgres::{types::Type, IsolationLevel};

use deadpool_postgres::{ManagerConfig, Pool, RecyclingMethod};

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    pg: deadpool_postgres::Config,
}

impl Config {
    pub fn from_env() -> Self {
        let mut cfg = ::config_crate::Config::new();
        cfg.merge(::config_crate::Environment::new().separator("__"))
            .unwrap();
        let mut cfg = cfg.try_into::<Self>().unwrap();
        cfg.pg.dbname.get_or_insert("deadpool".to_string());
        cfg
    }
    pub fn from_env_with_prefix(prefix: &str) -> Self {
        let mut cfg = ::config_crate::Config::new();
        cfg.merge(::config_crate::Environment::with_prefix(prefix).separator("__"))
            .unwrap();
        let mut cfg = cfg.try_into::<Self>().unwrap();
        cfg.pg.dbname.get_or_insert("deadpool".to_string());
        cfg
    }
}

fn create_pool() -> Pool<tokio_postgres::NoTls> {
    let cfg = Config::from_env();
    cfg.pg.create_pool(tokio_postgres::NoTls).unwrap()
}

#[tokio::test]
async fn test_basic() {
    let pool = create_pool();
    let client = pool.get().await.unwrap();
    let stmt = client.prepare("SELECT 1 + 2").await.unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();
    let value: i32 = rows[0].get(0);
    assert_eq!(value, 3);
    assert_eq!(client.statement_cache.size(), 1);
}

#[tokio::test]
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

#[tokio::test]
async fn test_prepare_typed_error() {
    let pool = create_pool();
    let client = pool.get().await.unwrap();
    let stmt = client
        .prepare_typed("SELECT 1 + $1", &[Type::INT2])
        .await
        .unwrap();
    assert!(client.query(&stmt, &[&42i32]).await.is_err());
}

#[tokio::test]
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

#[tokio::test]
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

#[tokio::test]
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

#[tokio::test]
async fn test_transaction_builder() {
    let pool = create_pool();
    let mut client = pool.get().await.unwrap();
    let txn = client
        .build_transaction()
        .isolation_level(IsolationLevel::ReadUncommitted)
        .read_only(true)
        .deferrable(true)
        .start()
        .await
        .unwrap();
    let rows = txn.query("SELECT 1 + 2", &[]).await.unwrap();
    let value: i32 = rows[0].get(0);
    assert_eq!(value, 3);
    txn.commit().await.unwrap();
}

#[tokio::test]
async fn test_generic_client() {
    let pool = create_pool();
    let client = pool.get().await.unwrap();
    _use_generic_client(&**client);
}

#[tokio::test]
async fn test_recycling_methods() {
    let recycling_methods = vec![
        RecyclingMethod::Fast,
        RecyclingMethod::Verified,
        RecyclingMethod::Clean,
        RecyclingMethod::Custom("DISCARD ALL;".to_string()),
    ];
    let mut cfg = Config::from_env();
    for recycling_method in recycling_methods {
        cfg.pg.manager = Some(ManagerConfig { recycling_method });
        let pool = cfg.pg.create_pool(tokio_postgres::NoTls).unwrap();
        for _ in 0usize..20usize {
            let client = pool.get().await.unwrap();
            let rows = client.query("SELECT 1 + 2", &[]).await.unwrap();
            let value: i32 = rows[0].get(0);
            assert_eq!(value, 3);
        }
    }
}

fn _use_generic_client(_client: &impl tokio_postgres::GenericClient) {
    // nop
}

struct Env {
    backup: HashMap<String, Option<String>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            backup: HashMap::new(),
        }
    }
    pub fn set(&mut self, name: &str, value: &str) {
        self.backup.insert(name.to_string(), env::var(name).ok());
        env::set_var(name, value);
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        for (name, value) in self.backup.iter() {
            println!("setting {} = {:?}", name, value);
            match value {
                Some(value) => env::set_var(name.as_str(), value),
                None => env::remove_var(name.as_str()),
            }
        }
    }
}

#[test]
#[cfg(feature = "config")]
fn test_config_from_env() {
    // This test must not use "PG" as prefix as this can cause the other
    // tests which also use the "PG" prefix to fail.
    let mut env = Env::new();
    env.set("ENV_TEST_PG__HOST", "pg.example.com");
    env.set("ENV_TEST_PG__PORT", "5433");
    env.set("ENV_TEST_PG__USER", "john_doe");
    env.set("ENV_TEST_PG__PASSWORD", "topsecret");
    env.set("ENV_TEST_PG__DBNAME", "example");
    env.set("ENV_TEST_PG__POOL__MAX_SIZE", "42");
    env.set("ENV_TEST_PG__POOL__TIMEOUTS__WAIT__SECS", "1");
    env.set("ENV_TEST_PG__POOL__TIMEOUTS__WAIT__NANOS", "0");
    env.set("ENV_TEST_PG__POOL__TIMEOUTS__CREATE__SECS", "2");
    env.set("ENV_TEST_PG__POOL__TIMEOUTS__CREATE__NANOS", "0");
    env.set("ENV_TEST_PG__POOL__TIMEOUTS__RECYCLE__SECS", "3");
    env.set("ENV_TEST_PG__POOL__TIMEOUTS__RECYCLE__NANOS", "0");
    let cfg = Config::from_env_with_prefix("ENV_TEST");
    // `tokio_postgres::Config` does not provide any read access to its
    // internals so we can only check if the environment was actually read
    // correctly.
    assert_eq!(cfg.pg.host, Some("pg.example.com".to_string()));
    assert_eq!(cfg.pg.port, Some(5433));
    assert_eq!(cfg.pg.user, Some("john_doe".to_string()));
    assert_eq!(cfg.pg.password, Some("topsecret".to_string()));
    assert_eq!(cfg.pg.dbname, Some("example".to_string()));
    let pool_cfg = cfg.pg.get_pool_config();
    assert_eq!(pool_cfg.max_size, 42);
    assert_eq!(pool_cfg.timeouts.wait, Some(Duration::from_secs(1)));
    assert_eq!(pool_cfg.timeouts.create, Some(Duration::from_secs(2)));
    assert_eq!(pool_cfg.timeouts.recycle, Some(Duration::from_secs(3)));
}
