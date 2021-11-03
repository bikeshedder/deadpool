use serde_1::Deserialize;

use deadpool_arangodb::{Pool, Runtime};


fn default_dbname() -> String {
    "deadpool".to_string()
}


#[derive(Debug, Default, Deserialize)]
#[serde(crate = "serde_1")]
struct Config {
    #[serde(default)]
    arango: deadpool_arangodb::Config,
    #[serde(default="default_dbname")]
    dbname: String,
}

impl Config {
    pub fn from_env() -> Self {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new().separator("__"))
            .unwrap();
        let mut cfg = cfg.try_into::<Self>().unwrap();
        cfg.arango.url.get_or_insert("http://localhost:8529".to_string());
        cfg
    }
}

#[tokio::test]
async fn create_database() {
    let cfg = Config::from_env();
    let pool = cfg.arango.create_pool(Runtime::Tokio1).unwrap();
    let conn = pool.get().await.unwrap();

    let result = conn.create_database(&cfg.dbname).await;
    if let Err(e) = result {
        assert!(false, "Failed to create database: {:?}", e)
    };
    let result = conn.db(&cfg.dbname).await;
    assert!(result.is_ok());

    let result = conn.drop_database(&cfg.dbname).await;
    if let Err(e) = result {
        assert!(false, "Failed to drop database: {:?}", e)
    };
    let result = conn.db(&cfg.dbname).await;
    assert!(result.is_err());
}
