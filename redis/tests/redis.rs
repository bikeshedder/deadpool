#![cfg(feature = "serde")]

use deadpool_redis::Runtime;
use futures::FutureExt;
use redis::cmd;
use serde_1::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(crate = "serde_1")]
struct Config {
    #[serde(default)]
    redis: deadpool_redis::Config,
}

impl Config {
    pub fn from_env() -> Self {
        let mut cfg = config::Config::default();
        cfg.merge(config::Environment::new().separator("__"))
            .unwrap();
        cfg.try_into().unwrap()
    }
}

fn create_pool() -> deadpool_redis::Pool {
    let cfg = Config::from_env();
    cfg.redis.create_pool(Some(Runtime::Tokio1)).unwrap()
}

#[tokio::test]
async fn test_pipeline() {
    use deadpool_redis::redis::pipe;
    let pool = create_pool();
    let mut conn = pool.get().await.unwrap();
    let (value,): (String,) = pipe()
        .cmd("SET")
        .arg("deadpool/pipeline_test_key")
        .arg("42")
        .ignore()
        .cmd("GET")
        .arg("deadpool/pipeline_test_key")
        .query_async(&mut conn)
        .await
        .unwrap();
    assert_eq!(value, "42".to_string());
}

#[tokio::test]
async fn test_high_level_commands() {
    use deadpool_redis::redis::AsyncCommands;
    let pool = create_pool();
    let mut conn = pool.get().await.unwrap();
    let _: () = conn.set("deadpool/hlc_test_key", 42).await.unwrap();
    let value: isize = conn.get("deadpool/hlc_test_key").await.unwrap();
    assert_eq!(value, 42);
}

#[tokio::test]
async fn test_aborted_command() {
    let pool = create_pool();

    {
        let mut conn = pool.get().await.unwrap();
        // Poll the future once. This does execute the query but does not
        // wait for the response. The connection now has a request queued
        // and the response to it will be returned when using the connection
        // the next time:
        // https://github.com/bikeshedder/deadpool/issues/97
        // https://github.com/mitsuhiko/redis-rs/issues/489
        cmd("PING")
            .arg("wrong")
            .query_async::<_, String>(&mut conn)
            .now_or_never();
    }
    {
        let mut conn = pool.get().await.unwrap();
        let value: String = cmd("PING")
            .arg("right")
            .query_async(&mut conn)
            .await
            .unwrap();
        assert_eq!(value, "right");
    }
}

#[tokio::test]
async fn test_recycled() {
    let pool = create_pool();

    let client_id = {
        let mut conn = pool.get().await.unwrap();

        cmd("CLIENT")
            .arg("ID")
            .query_async::<_, i64>(&mut conn)
            .await
            .unwrap()
    };

    {
        let mut conn = pool.get().await.unwrap();

        let new_client_id = cmd("CLIENT")
            .arg("ID")
            .query_async::<_, i64>(&mut conn)
            .await
            .unwrap();

        assert_eq!(
            client_id, new_client_id,
            "the redis connection was not recycled"
        );
    }
}
