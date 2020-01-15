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
use std::env;

use deadpool_redis::Config;
use redis::FromRedisValue;

#[tokio::main]
async fn main() {
    let cfg = Config::from_env("REDIS").unwrap();
    let pool = cfg.create_pool().unwrap();
    {
        let mut conn = pool.get().await.unwrap();
        let mut cmd = redis::cmd("SET");
        cmd.arg(&["deadpool/test_key", "42"]);
        conn.query(&cmd).await.unwrap();
    }
    {
        let mut conn = pool.get().await.unwrap();
        let mut cmd = redis::cmd("GET");
        cmd.arg(&["deadpool/test_key"]);
        let value = conn.query(&cmd).await.unwrap();
        assert_eq!(String::from_redis_value(&value).unwrap(), "42".to_string());
    }
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
