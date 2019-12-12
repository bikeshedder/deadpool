#[tokio::main]
#[test]
async fn test_pipeline() {
    use deadpool_redis::{pipe, Manager, Pool};
    let mgr = Manager::new("redis://127.0.0.1/").unwrap();
    let pool = Pool::new(mgr, 16);
    let mut conn = pool.get().await.unwrap();
    let (value,): (String,) = pipe()
        .cmd("SET")
        .arg("deadpool/pipeline_test_key")
        .arg("42")
        .ignore()
        .cmd("GET")
        .arg("deadpool/pipeline_test_key")
        .query(&mut conn)
        .await
        .unwrap();
    assert_eq!(value, "42".to_string());
}
