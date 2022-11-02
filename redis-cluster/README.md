# Deadpool for Redis Cluster [![Latest Version](https://img.shields.io/crates/v/deadpool-redis-cluster.svg)](https://crates.io/crates/deadpool-redis-cluster) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.54+](https://img.shields.io/badge/rustc-1.54+-lightgray.svg "Rust 1.54+")](https://blog.rust-lang.org/2021/07/29/Rust-1.54.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`redis-cluster`](https://crates.io/crates/redis_cluster_async).

## Features

| Feature          | Description                                                           | Extra dependencies                                | Default |
| ---------------- | --------------------------------------------------------------------- | ------------------------------------------------- | ------- |
| `rt_tokio_1`     | Enable support for [tokio](https://crates.io/crates/tokio) crate      | `deadpool/rt_tokio_1`, `redis/tokio-comp`         | yes     |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate | `deadpool/rt_async-std_1`, `redis/async-std-comp` | no      |
| `serde`          | Enable support for [serde](https://crates.io/crates/serde) crate      | `deadpool/serde`, `serde/derive`                  | no      |

## Example

```rust
use deadpool_redis_cluster::{redis::{cmd, FromRedisValue}, Config, Runtime};

#[tokio::main]
async fn main() {
    let mut cfg = Config::from_urls(vec![
        "redis://127.0.0.1:7000".to_string(),
        "redis://127.0.0.1:7001".to_string(),
    ]);
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&["deadpool/test_key", "42"])
            .query_async::<_, ()>(&mut conn)
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
use deadpool_redis_cluster::{redis::{cmd, FromRedisValue}, Runtime};
use dotenv::dotenv;
# use serde_1 as serde;

#[derive(Debug, serde::Deserialize)]
# #[serde(crate = "serde_1")]
struct Config {
    #[serde(default)]
    redis_cluster: deadpool_redis_cluster::Config
}

impl Config {
      pub fn from_env() -> Result<Self, config::ConfigError> {
         config::Config::builder()
            .add_source(
                config::Environment::default()
                .separator("__")
                .try_parsing(true)
                .list_separator(","),
            )
            .build()?
            .try_deserialize()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cfg = Config::from_env().unwrap();
    let pool = cfg.redis_cluster.create_pool(Some(Runtime::Tokio1)).unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        cmd("SET")
            .arg(&["deadpool/test_key", "42"])
            .query_async::<_, ()>(&mut conn)
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
  `deadpool-redis-cluster` does and enable the needed features in your own
  `Crate.toml` file:

  ```toml
  [dependencies]
  deadpool-redis-cluster = { version = "0.9", features = ["serde"] }
  redis = { version = "0.21", default-features = false, features = ["tls"] }
  ```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
