use actix_web::{error, get, web, App, HttpResponse, HttpServer};
use config::ConfigError;
use deadpool_postgres::{Client, Pool, PoolError};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Config {
    listen: String,
    pg: deadpool_postgres::Config,
}

impl Config {
    fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = ::config::Config::new();
        cfg.merge(::config::Environment::new().separator("__"))?;
        cfg.try_into()
    }
}

#[derive(Serialize, Deserialize)]
struct Event {
    id: Uuid,
    title: String,
}

#[derive(failure::Fail, Debug)]
enum Error {
    #[fail(display = "An internal error occured. Please try again later.")]
    PoolError(PoolError),
}

impl From<PoolError> for Error {
    fn from(error: PoolError) -> Self {
        Self::PoolError(error)
    }
}

impl error::ResponseError for Error {}

async fn event_list(pool: &Pool) -> Result<Vec<Event>, PoolError> {
    let client: Client = pool.get().await?;
    let stmt = client.prepare("SELECT id, title FROM event").await?;
    let rows = client.query(&stmt, &[]).await?;
    Ok(rows
        .into_iter()
        .map(|row| Event {
            id: row.get(0),
            title: row.get(1),
        })
        .collect())
}

#[get("/v1.0/event.list")]
async fn index(db_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let events = event_list(&db_pool).await?;
    Ok(HttpResponse::Ok().json(events))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let config = Config::from_env().unwrap();
    let pool = config.pg.create_pool(tokio_postgres::NoTls).unwrap();
    let server = HttpServer::new(move || App::new().data(pool.clone()).service(index))
        .bind(&config.listen)?
        .run();
    println!("Server running at http://{}/", &config.listen);
    println!(
        "Try the following URLs: http://{}/v1.0/event.list",
        &config.listen,
    );
    server.await
}
