# Deadpool for Lapin [![Latest Version](https://img.shields.io/crates/v/deadpool-lapin.svg)](https://crates.io/crates/deadpool-lapin)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`lapin`](https://crates.io/crates/lapin).

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |

## Example with `tokio-amqp` crate

```rust
use std::sync::Arc;

use deadpool_lapin::{Config, Manager, Pool };
use deadpool_lapin::lapin::{
    options::BasicPublishOptions,
    BasicProperties
};
use tokio::runtime::Runtime;
use tokio_amqp::LapinTokioExt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Arc::new(Runtime::new()?);
    let mut cfg = Config::default();
    cfg.url = Some("amqp://localhost/%2f".to_string());
    cfg.connection_properties = lapin::ConnectionProperties::default()
            .with_tokio(rt.clone());
    let pool = cfg.create_pool();
    rt.block_on(async move {
        for i in 1..10usize {
            let mut connection = pool.get().await?;
            let channel = connection.create_channel().await?;
            channel.basic_publish(
                "",
                "hello",
                BasicPublishOptions::default(),
                b"hello from deadpool".to_vec(),
                BasicProperties::default()
            ).await?;
        }
        Ok(())
    })
}
```

## Example with `config`, `dotenv` and `tokio-amqp` crate

```rust
use std::sync::Arc;

use deadpool_lapin::lapin::{
    options::BasicPublishOptions,
    BasicProperties
};
use dotenv::dotenv;
use serde::Deserialize;
use tokio::runtime::Runtime;
use tokio_amqp::LapinTokioExt;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let rt = Arc::new(Runtime::new()?);
    let mut cfg = Config::from_env().unwrap();
    cfg.amqp.connection_properties = lapin::ConnectionProperties::default()
            .with_tokio(rt.clone());
    let pool = cfg.amqp.create_pool();
    rt.block_on(async move {
        for i in 1..10usize {
            let mut connection = pool.get().await?;
            let channel = connection.create_channel().await?;
            channel.basic_publish(
                "",
                "hello",
                BasicPublishOptions::default(),
                b"hello from deadpool".to_vec(),
                BasicProperties::default()
            ).await?;
        }
        Ok(())
    })
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
