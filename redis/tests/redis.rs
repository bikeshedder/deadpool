fn create_pool() -> deadpool_redis::Pool {
    use deadpool_redis::Config;
    let cfg = Config::from_env("REDIS").unwrap();
    let pool = cfg.create_pool().unwrap();
    pool
}

#[tokio::main]
#[test]
async fn test_pipeline() {
    use deadpool_redis::pipe;
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

#[tokio::main]
#[test]
async fn test_high_level_commands() {
    use redis::AsyncCommands;
    let pool = create_pool();
    let mut conn = pool.get().await.unwrap();
    let _: () = conn.set("deadpool/hlc_test_key", 42).await.unwrap();
    let value: isize = conn.get("deadpool/hlc_test_key").await.unwrap();
    assert_eq!(value, 42);
}