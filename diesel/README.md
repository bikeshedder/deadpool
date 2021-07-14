# Deadpool for Diesel [![Latest Version](https://img.shields.io/crates/v/deadpool-diesel.svg)](https://crates.io/crates/deadpool-diesel)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`diesel`](https://crates.io/crates/diesel) connections.

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `sqlite` | Enable `sqlite` feature in `diesel` crate | `diesel/sqlite` | no |
| `postgres` | Enable `postgres` feature in `diesel` crate | `diesel/postgres` | no |
| `mysql` | Enable `mysql` feature in `diesel` crate | `diesel/mysql` | no |
| `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate | `deadpool/rt_async-std_1` | no |

## Example

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = deadpool_diesel::sqlite::Manager::new(":memory:");
    let pool = deadpool_diesel::sqlite::Pool::new(manager, 8);
    let conn = pool.get().await?;
    // TODO use the connection with diesel
    Ok(())
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
