# Deadpool for PostgreSQL [![Latest Version](https://img.shields.io/crates/v/deadpool-postgres.svg)](https://crates.io/crates/deadpool-postgres)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`tokio-postgres`](https://crates.io/crates/tokio-postgres)
and also provides a `statement` cache by wrapping `tokio_postgres::Client`
and `tokio_postgres::Transaction`.

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |

## Example

```rust
use deadpool_sqlite::Config;

#[tokio::main]
async fn main() {
    let mut cfg = Config::new("db.sqlite3");
    let pool = cfg.create_pool();
    for i in 1..10 {
        let mut conn = pool.get().await.unwrap();
        let value: i32 = conn.interact(move |conn| {
            let mut stmt = conn.prepare_cached("SELECT 1 + $1").unwrap();
            stmt.query_row([&i], |row| row.get(0)).unwrap()
        }).await;
        assert_eq!(value, i + 1);
    }
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
