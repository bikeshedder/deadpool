# Deadpool for PostgreSQL [![Latest Version](https://img.shields.io/crates/v/deadpool-postgres.svg)](https://crates.io/crates/deadpool-postgres)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`tokio-postgres`](https://crates.io/crates/tokio-postgres)
and also provides a `statement` cache by wrapping `tokio_postgres::Client`
and `tokio_postgres::Transaction`.

## Example

```rust
use std::env;

use deadpool_postgres::{Manager, Pool};
use tokio_postgres::{Config, NoTls};

#[tokio::main]
async fn main() {
    let mut cfg = Config::new();
    cfg.host("/var/run/postgresql");
    cfg.user(env::var("USER").unwrap().as_str());
    cfg.dbname("deadpool");
    let mgr = Manager::new(cfg, tokio_postgres::NoTls);
    let pool = Pool::new(mgr, 16);
    for i in 1..10 {
        let mut client = pool.get().await.unwrap();
        let stmt = client.prepare("SELECT 1 + $1").await.unwrap();
        let rows = client.query(&stmt, &[&i]).await.unwrap();
        let value: i32 = rows[0].get(0);
        assert_eq!(value, i + 1);
    }
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
