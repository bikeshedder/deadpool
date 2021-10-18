# Deadpool for R2D2 Managers [![Latest Version](https://img.shields.io/crates/v/deadpool-r2d2.svg)](https://crates.io/crates/deadpool-r2d2) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.54+](https://img.shields.io/badge/rustc-1.54+-lightgray.svg "Rust 1.54+")](https://blog.rust-lang.org/2021/07/29/Rust-1.54.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`r2d2`](https://crates.io/crates/r2d2) managers.

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate | `deadpool/rt_async-std_1` | no |
| `serde` | Enable support for [serde](https://crates.io/crates/serde) crate | `deadpool/serde` | no |

## Example

```rust
use std::env;

use deadpool_r2d2::Runtime;

type PostgresManager = deadpool_r2d2::Manager<
    r2d2_postgres::PostgresConnectionManager<r2d2_postgres::postgres::NoTls>,
>;
type PostgresPool = deadpool_r2d2::Pool<PostgresManager>;

fn create_pool(max_size: usize) -> PostgresPool {
    let mut pg_config = r2d2_postgres::postgres::Config::new();
    pg_config.host_path("/run/postgresql");
    pg_config.user(&env::var("USER").unwrap());
    pg_config.dbname("deadpool");
    let r2d2_manager =
        r2d2_postgres::PostgresConnectionManager::new(pg_config, r2d2_postgres::postgres::NoTls);
    let manager = PostgresManager::new(r2d2_manager, Runtime::Tokio1);
    let pool = PostgresPool::builder(manager)
        .max_size(max_size)
        .build()
        .unwrap();
    pool
}

#[tokio::main]
async fn main() {
    let pool = create_pool(2);
    let client = pool.get().await.unwrap();
    let answer: i32 = client
        .interact(|client| {
            let row = client.query_one("SELECT 42", &[])?;
            Ok(row.get(0))
        })
        .await
        .unwrap();
    assert_eq!(answer, 42);
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
