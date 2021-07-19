# Deadpool for SQLite [![Latest Version](https://img.shields.io/crates/v/deadpool-sqlite.svg)](https://crates.io/crates/deadpool-sqlite)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`rusqlite`](https://crates.io/crates/rusqlite)
and provides a wrapper that ensures correct use of the connection
inside a separate thread.

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |
| `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate | `deadpool/rt_async-std_1` | no |

## Example

```rust
use deadpool_sqlite::{Config, Runtime};

#[tokio::main]
async fn main() {
    let mut cfg = Config::new("db.sqlite3");
    let pool = cfg.create_pool(Runtime::Tokio1);
    let conn = pool.get().await.unwrap();
    let result: i64 = conn
        .interact(|conn| {
            let mut stmt = conn.prepare("SELECT 1")?;
            let mut rows = stmt.query([])?;
            let row = rows.next()?.unwrap();
            row.get(0)
        })
        .await
        .unwrap();
    assert_eq!(result, 1);
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
