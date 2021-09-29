use std::env;

use actix_web::{error, get, middleware, web, App, Error, HttpResponse, HttpServer};
use deadpool_redis::{redis::cmd, Config as RedisConfig, Connection, Pool, PoolError, Runtime};

fn redis_uri() -> String {
    match env::var("REDIS_URL") {
        Ok(s) if !s.is_empty() => s,
        _ => "redis://127.0.0.1:6379".into(),
    }
}

async fn redis_ping(pool: &Pool) -> Result<String, PoolError> {
    let mut conn: Connection = pool.get().await?;
    let pong: String = cmd("PING").query_async(&mut conn).await?;

    Ok(pong)
}

#[get("/")]
async fn index(redis_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let pong = redis_ping(&redis_pool)
        .await
        .map_err(|pool_error| error::ErrorNotAcceptable(format!("{}", pool_error)))?;

    Ok(HttpResponse::Ok().body(format!("Redis PING -> {}", pong)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let redis_config = RedisConfig::from_url(redis_uri());
    let redis_pool = redis_config.create_pool(Some(Runtime::Tokio1)).unwrap();
    let server_url = "127.0.0.1:8080";

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis_pool.clone()))
            .wrap(middleware::Logger::default())
            .service(index)
    })
    .bind(server_url)?
    .run();

    println!(
        "Server running! Access the index page here: http://{}/",
        server_url
    );

    server.await
}
