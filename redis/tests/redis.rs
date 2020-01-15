#[tokio::main]
#[test]
async fn test_pipeline() {
    use deadpool_redis::{pipe, Config};
    let cfg = Config::from_env("REDIS").unwrap();
    let pool = cfg.create_pool().unwrap();
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
