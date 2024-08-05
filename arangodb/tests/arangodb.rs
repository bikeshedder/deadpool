use config::ConfigError;
use serde::Deserialize;

use deadpool_arangodb::Runtime;

fn default_dbname() -> String {
    "deadpool".to_string()
}

#[derive(Debug, Default, Deserialize)]
struct Config {
    #[serde(default)]
    arango: deadpool_arangodb::Config,
    #[serde(default = "default_dbname")]
    dbname: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let cfg = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()
            .unwrap();
        let mut cfg: Self = cfg.try_deserialize()?;
        cfg.arango
            .url
            .get_or_insert("http://localhost:8529".to_string());
        Ok(cfg)
    }
}

#[tokio::test]
async fn create_database() {
    let cfg = Config::from_env().unwrap();
    let pool = cfg.arango.create_pool(Runtime::Tokio1).unwrap();
    let conn = pool.get().await.unwrap();

    let result = conn.create_database(&cfg.dbname).await;
    if let Err(e) = result {
        panic!("Failed to create database: {:?}", e)
    };
    let result = conn.db(&cfg.dbname).await;
    assert!(result.is_ok());

    let result = conn.drop_database(&cfg.dbname).await;
    if let Err(e) = result {
        panic!("Failed to drop database: {:?}", e)
    };
    let result = conn.db(&cfg.dbname).await;
    assert!(result.is_err());
}
