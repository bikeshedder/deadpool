use deadpool_diesel::postgres::{BuildError, Manager, Pool};

use deadpool_diesel::Runtime;

use std::env;
use thiserror::Error;

#[derive(Error, Debug)]

pub enum PoolError {
    #[error("unable to load .env file")]
    Env(dotenvy::Error),

    #[error("missing DATABASE_URL")]
    DatabaseURL,

    #[error("unable to build pool")]
    PoolBuildError(BuildError),
}

pub fn set_up_pool() -> Result<Pool, PoolError> {
    dotenvy::dotenv().map_err(PoolError::Env)?;

    let database_url = env::var("DATABASE_URL").map_err(|_| PoolError::DatabaseURL)?;

    let manager = Manager::new(database_url, Runtime::Tokio1);

    let pool = Pool::builder(manager)
        .max_size(8)
        .build()
        .map_err(PoolError::PoolBuildError)?;

    Ok(pool)
}

pub fn main() {}
