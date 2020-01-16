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

```rust
use deadpool_redis::{cmd, Config};
use redis::FromRedisValue;

#[tokio::main]
async fn main() {
    let cfg = Config::from_env("REDIS").unwrap();
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

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
