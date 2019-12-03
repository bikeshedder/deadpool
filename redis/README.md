# Deadpool for Redis [![Latest Version](https://img.shields.io/crates/v/deadpool-redis.svg)](https://crates.io/crates/deadpool-redis)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`redis`](https://crates.io/crates/redis).

## Example

```rust
use std::env;

use deadpool_redis::{Manager, Pool};
use futures::compat::Future01CompatExt;
use redis::FromRedisValue;

#[tokio::main]
async fn main() {
    let mgr = Manager::new("redis://127.0.0.1/").unwrap();
    let pool = Pool::new(mgr, 16);
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

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
