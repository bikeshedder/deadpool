use std::fmt;
use std::net::SocketAddr;

use config::ConfigError;
use deadpool_postgres::{Client, Pool, PoolError};
use dotenv::dotenv;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Server, StatusCode};
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

#[derive(Debug)]
enum Error {
    PoolError(PoolError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PoolError(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<PoolError> for Error {
    fn from(error: PoolError) -> Self {
        Self::PoolError(error)
    }
}

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

async fn handle(req: Request<Body>, pool: Pool) -> Result<Response<Body>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/v1.0/event.list") => {
            let events = event_list(&pool).await?;
            let json = serde_json::to_string(&events).unwrap();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json))
                .unwrap();
            Ok(response)
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    async {
        dotenv().ok();
        let config = Config::from_env()?;
        let addr: SocketAddr = config.listen.parse()?;
        let pool = config.pg.create_pool(tokio_postgres::NoTls)?;

        let make_svc = make_service_fn(|_conn| {
            let pool = pool.clone();
            async { Ok::<_, Error>(service_fn(move |req| handle(req, pool.clone()))) }
        });

        let server = Server::bind(&addr).serve(make_svc);

        println!("Server running at http://{}/", &config.listen);
        println!(
            "Try the following URLs: http://{}/v1.0/event.list",
            &config.listen
        );

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }

        Ok(())
    }
    .await
}
