# Deadpool for Lapin [![Latest Version](https://img.shields.io/crates/v/deadpool-lapin.svg)](https://crates.io/crates/deadpool-lapin)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`lapin`](https://crates.io/crates/lapin).

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |

## Example

```rust,ignore
use deadpool_lapin::{Config, Manager, Pool };
use deadpool_lapin::lapin::{
    options::BasicPublishOptions,
    BasicProperties
};

#[tokio::main]
async fn main() {
    let mut cfg = Config::default();
    cfg.url = Some("amqp://localhost/%2f".to_string());
    let pool = cfg.create_pool();
    for i in 1..10usize {
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

## Example with `config` and `dotenv` crate

```rust
use deadpool_lapin::lapin::{
    options::BasicPublishOptions,
    BasicProperties
};
use dotenv::dotenv;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    amqp: deadpool_lapin::Config
}

impl Config {
    pub fn from_env() -> Result<Self, ::config_crate::ConfigError> {
        let mut cfg = ::config_crate::Config::new();
        cfg.merge(::config_crate::Environment::new().separator("__"))?;
        cfg.try_into()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cfg = Config::from_env().unwrap();
    let pool = cfg.amqp.create_pool();
    for i in 1..10usize {
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
