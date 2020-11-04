# Deadpool for Redis [![Latest Version](https://img.shields.io/crates/v/deadpool-redis.svg)](https://crates.io/crates/deadpool-redis)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`redis`](https://crates.io/crates/redis).

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |

## Example

```rust,ignore
use deadpool_redis::{cmd, Config};
use deadpool_redis::redis::FromRedisValue;

#[tokio::main]
async fn main() {
    let mut cfg = Config::default();
    cfg.url = Some("redis://127.0.0.1/".to_string());
    let pool = cfg.create_pool().unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&["deadpool/test_key", "42"])
            .execute_async(&mut conn)
            .await.unwrap();
    }
    {
        let mut conn = pool.get().await.unwrap();
        let value: String = cmd("GET")
            .arg(&["deadpool/test_key"])
            .query_async(&mut conn)
            .await.unwrap();
        assert_eq!(value, "42".to_string());
    }
}
```

## Example with `config` and `dotenv` crate

```rust
use deadpool_redis::cmd;
use deadpool_redis::redis::FromRedisValue;
use dotenv::dotenv;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default)]
    redis: deadpool_redis::Config
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
    let pool = cfg.redis.create_pool().unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&["deadpool/test_key", "42"])
            .execute_async(&mut conn)
            .await.unwrap();
    }
    {
        let mut conn = pool.get().await.unwrap();
        let value: String = cmd("GET")
            .arg(&["deadpool/test_key"])
            .query_async(&mut conn)
            .await.unwrap();
        assert_eq!(value, "42".to_string());
    }
}
```

## FAQ

- **How can I enable features of the `redis` crate?**

  Make sure that you depend on the same version of `redis` as
  `deadpool-redis` does and enable the needed features in your own
  `Crate.toml` file:

  ```toml
  [dependencies]
  deadpool-redis = { version = "0.6", features = ["config"] }
  redis = { version = "0.17", default-features = false, features = ["tls"] }
  ```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
