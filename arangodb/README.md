# Deadpool for ArangoDB [![Latest Version](https://img.shields.io/crates/v/deadpool-arangodb.svg)](https://crates.io/crates/deadpool-arangodb) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.54+](https://img.shields.io/badge/rustc-1.54+-lightgray.svg "Rust 1.54+")](https://blog.rust-lang.org/2021/07/29/Rust-1.54.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`ArangoDB`](https://www.arangodb.com/) using [`arangors`](https://crates.io/crates/arangors).

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate,<br>through the usage of [reqwest](https://crates.io/crates/reqwest) as http client | `deadpool/rt_tokio_1`, `arangors/reqwest_async`  | yes |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate,<br>through the usage of [surf](https://crates.io/crates/surf) as http client | `deadpool/rt_async-std_1`, `arangors/surf_async`  | no |
| `serde` | Enable support for [serde](https://crates.io/crates/serde) crate | `deadpool/serde`, `serde/derive` | no |

## Example

```rust
use deadpool_arangodb::{Config, Runtime};

#[tokio::main]
async fn main() {
    let mut cfg = Config {
        url: Some("http://localhost:8529".to_string()),
        username: Some("root".to_string()),
        password: Some("deadpool".to_string()),
        use_jwt: true,
        pool: None,
    };
    let pool = cfg.create_pool(Runtime::Tokio1).unwrap();
    let mut conn = pool.get().await.unwrap();

    let db = conn.create_database("deadpool_favorite_foods")
        .await.expect("Failed to create database: {:?}");

    // Do stuff with db...
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
