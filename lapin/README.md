# Deadpool for Lapin [![Latest Version](https://img.shields.io/crates/v/deadpool-lapin.svg)](https://crates.io/crates/deadpool-lapin)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`lapin`](https://crates.io/crates/lapin).

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |
| `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate | `deadpool/rt_async-std_1` | no |

## Example with `tokio-amqp` crate

```rust,ignore
use std::sync::Arc;

use deadpool_lapin::{Config, Manager, Pool };
use deadpool_lapin::lapin::{
    options::BasicPublishOptions,
    BasicProperties
};
use tokio::runtime::Runtime;
use tokio_amqp::LapinTokioExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Config::default();
    cfg.url = Some("amqp://127.0.0.1:5672/%2f".to_string());
    cfg.pool.get_or_insert_with(Default::default).runtime = deadpool::Runtime::Tokio1;
    let pool = cfg.create_pool();
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let mut cfg = Config::from_env().unwrap();
    cfg.amqp.pool.get_or_insert_with(Default::default).runtime = deadpool::Runtime::Tokio1;
    let pool = cfg.amqp.create_pool();
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
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
