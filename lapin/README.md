# Deadpool for Lapin [![Latest Version](https://img.shields.io/crates/v/deadpool-lapin.svg)](https://crates.io/crates/deadpool-lapin)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`lapin`](https://crates.io/crates/lapin).

This crate depends on the current git version which adds `async/.await` support and is therefore considered an alpha version.

## Example

```rust
use std::env;

use deadpool_lapin::{Manager, Pool};
use lapin::{
    ConnectionProperties,
    options::BasicPublishOptions,
    BasicProperties
};

#[tokio::main]
async fn main() {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(
        |_| "amqp://127.0.0.1:5672/%2f".into());
    let mgr = Manager::new(addr, ConnectionProperties::default());
    let pool = Pool::new(mgr, 16);
    for i in 1..10 {
        let mut connection = pool.get().await.unwrap();
        let channel = connection.create_channel().await.unwrap();
        channel.basic_publish(
            "",
            "hello",
            BasicPublishOptions::default(),
            b"hello from deadpool".to_vec(),
            BasicProperties::default()
        ).await.unwrap();
    }
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
