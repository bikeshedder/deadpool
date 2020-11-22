use actix_web::{error, get, middleware, web, App, HttpServer, HttpResponse, Error};
use deadpool_redis::{cmd, Connection, Pool, PoolError, Config as RedisConfig};
use std::env;


fn redis_uri() -> String {
    match env::var("REDIS_URL") {
        Ok(s) if !s.is_empty() => s,
        _ => String::from(String::from("redis://127.0.0.1:6379"))
    }
}

async fn redis_ping(pool: &Pool) -> Result<String, PoolError> {
    let mut connection: Connection = pool.get().await?;
    let pong: String = cmd("PING").query_async(&mut connection).await?;

    Ok(pong)
}

#[get("/")]
async fn index(redis_pool: web::Data<Pool>) -> Result<HttpResponse, Error> {
    let pong = redis_ping(&redis_pool).await.or_else(|pool_error| Err(error::ErrorNotAcceptable(format!("{}", pool_error))))?;

    Ok(HttpResponse::Ok().body(format!("Redis PING -> {}", pong)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let redis_config = RedisConfig { url: Some(redis_uri()), pool: None };
    let redis_pool = redis_config.create_pool().unwrap();
    let server_url = "127.0.0.1:8080";

    let server = HttpServer::new(move || {
        App::new()
            .data(redis_pool.clone())
            .wrap(middleware::Logger::default())
            .service(index)
    }).bind(server_url)?.run();

    println!("Server running! Access the index page here: http://{}/", server_url);

    server.await
}
