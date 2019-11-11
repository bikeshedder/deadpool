# Deadpool

Deadpool is a dead simple async pool for connections and objects
of any type.

## Features

Feature             | Description
------------------- | --------------------------------------
`postgres`          | Enable support for `tokio-postgres` connection
                    | pooling. This feature also includes a `statement`
                    | cache.

## Example

```rust
use std::env;

use deadpool::Pool;
use deadpool::postgres::Manager as PgManager;
use tokio_postgres::Config;

#[tokio::main]
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut cfg = tokio_postgres::Config::new();
    cfg.host("/var/run/postgresql");
    cfg.user(env::var("USER").unwrap().as_str());
    cfg.dbname("deadpool");
    let mgr = PgManager::new(cfg);
    let pool = Pool::new(mgr, 16);
    loop {
        let mut client = pool.get().await.unwrap();
        let stmt = client.prepare("SELECT random()").await.unwrap();
        let rows = client.query(&stmt, &[]).await.unwrap();
        let value: f64 = rows[0].get(0);
        println!("{}", value);
    }
}
```
