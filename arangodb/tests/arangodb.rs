use serde_1::Deserialize;

use deadpool_arangodb::{Pool, Runtime};


#[derive(Debug, Default, Deserialize)]
#[serde(crate = "serde_1")]
struct Config {
    #[serde(default)]
    arango: deadpool_arangodb::Config,
}

impl Config {
    pub fn from_env() -> Self {
        let mut cfg = Config::test_default();
        cfg.merge(config::Environment::new().separator("__"))
            .unwrap();
        cfg.try_into().unwrap()
    }

    pub fn test_default() -> Self {
        Self {
            arango: deadpool_arangodb::Config {
                url: Some("http://localhost:8529".to_string()),
                username: Some("root".to_string()),
                password: Some("deadpool".to_string()),
                use_jwt: true,
                pool: None,
            }
        }
    }
}

fn create_pool() -> Pool {
    let cfg = Config::test_default();
    cfg.arango.create_pool(Runtime::Tokio1).unwrap()
}

const DB_NAME: &str = "deadpool";

#[tokio::test]
async fn create_database() {
    let pool = create_pool();
    let conn = pool.get().await.unwrap();

    let result = conn.create_database(DB_NAME).await;
    if let Err(e) = result {
        assert!(false, "Failed to create database: {:?}", e)
    };
    let result = conn.db(DB_NAME).await;
    assert!(result.is_ok());

    let result = conn.drop_database(DB_NAME).await;
    if let Err(e) = result {
        assert!(false, "Failed to drop database: {:?}", e)
    };
    let result = conn.db(DB_NAME).await;
    assert!(result.is_err());
}
