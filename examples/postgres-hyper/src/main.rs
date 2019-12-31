use std::env;
use std::fmt;
use std::net::SocketAddr;

use deadpool_postgres::{Client, Manager, Pool, PoolError};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, header, Method, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};
use tokio_postgres::Config;
use uuid::Uuid;

const SERVER_ADDR: &str = "127.0.0.1:8080";

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
            Self::PoolError(err) => write!(f, "{}", err)
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
    let mut client: Client = pool.get().await?;
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

fn create_pool() -> Pool {
    let mut cfg = Config::new();
    cfg.host("/run/postgresql");
    cfg.user(env::var("USER").unwrap().as_str());
    cfg.dbname(
        env::var("PG_DBNAME")
            .expect("PG_DBNAME missing in environment")
            .as_str(),
    );
    let mgr = Manager::new(cfg, tokio_postgres::NoTls);
    Pool::new(mgr, 16)
}

#[tokio::main]
async fn main() {
    let addr: SocketAddr = SERVER_ADDR.parse().unwrap();
    let pool = create_pool();

    let make_svc = make_service_fn(|_conn| {
        let pool = pool.clone();
        async {
            Ok::<_, Error>(service_fn(move |req| handle(req, pool.clone())))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running at http://{}/", SERVER_ADDR);
    println!("Try the following URLs: http://{}/v1.0/event.list", SERVER_ADDR);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
