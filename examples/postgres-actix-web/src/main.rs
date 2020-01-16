use actix_web::{error, get, web, App, HttpResponse, HttpServer};
use config::ConfigError;
use deadpool_postgres::{Client, Config, Pool, PoolError};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const SERVER_ADDR: &str = "127.0.0.1:8080";

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

fn create_pool() -> Result<Pool, ConfigError> {
    let cfg = Config::from_env("PG")?;
    cfg.create_pool(tokio_postgres::NoTls)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pool = create_pool().unwrap();
    let server = HttpServer::new(move || App::new().data(pool.clone()).service(index))
        .bind(SERVER_ADDR)?
        .run();
    println!("Server running at http://{}/", SERVER_ADDR);
    println!(
        "Try the following URLs: http://{}/v1.0/event.list",
        SERVER_ADDR
    );
    server.await
}
