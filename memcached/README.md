# Deadpool for Memcached [![Latest Version](https://img.shields.io/crates/v/deadpool-memcached.svg)](https://crates.io/crates/deadpool-memcached)

Deadpool is a dead simple async pool for connections and objects of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`async-memcached`](https://crates.io/crates/async-memcached).

## Example

```rust,ignore
use deadpool_memcached::Manager;

#[tokio::main]
async fn main() {
    let manager = Manager::new("localhost:11211");
    let mut client = pool.get().await.unwrap();
    println!("version: {:?}", client.version().await);
}
```

## License

Licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>).
