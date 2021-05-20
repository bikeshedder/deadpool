# Deadpool for PostgreSQL [![Latest Version](https://img.shields.io/crates/v/deadpool-postgres.svg)](https://crates.io/crates/deadpool-postgres)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate implements a [`deadpool`](https://crates.io/crates/deadpool)
manager for [`tokio-postgres`](https://crates.io/crates/tokio-postgres)
and also provides a `statement` cache by wrapping `tokio_postgres::Client`
and `tokio_postgres::Transaction`.

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |
| `rt_tokio_1` | Enable support for [tokio](https://crates.io/crates/tokio) crate | `deadpool/rt_tokio_1` | yes |
| `rt_async-std_1` | Enable support for [async-std](https://crates.io/crates/config) crate | `deadpool/rt_async-std_1` | no |

**Important:** `async-std` support is currently limited to the
`async-std` specific timeout function. You still need to enable
the `tokio1` feature of `async-std` in order to use this crate
with `async-std`.

## Example

```rust,ignore
use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let mut cfg = Config::new();
    cfg.dbname = Some("deadpool".to_string());
    cfg.manager = Some(ManagerConfig { recycling_method: RecyclingMethod::Fast });
    let pool = cfg.create_pool(NoTls).unwrap();
    for i in 1..10 {
        let mut client = pool.get().await.unwrap();
        let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
        let rows = client.query(&stmt, &[&i]).await.unwrap();
        let value: i32 = rows[0].get(0);
        assert_eq!(value, i + 1);
    }
}
```

## Example with `config` and `dotenv` crate

```env
# .env
PG__DBNAME=deadpool
```

```rust
use deadpool_postgres::{Manager, Pool};
use dotenv::dotenv;
use serde::Deserialize;
use tokio_postgres::NoTls;

#[derive(Debug, Deserialize)]
struct Config {
    pg: deadpool_postgres::Config
}

impl Config {
    pub fn from_env() -> Result<Self, ::config_crate::ConfigError> {
        let mut cfg = ::config_crate::Config::new();
        cfg.set_default("pg.dbname", "deadpool");
        cfg.merge(::config_crate::Environment::new().separator("__"))?;
        cfg.try_into()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mut cfg = Config::from_env().unwrap();
    let pool = cfg.pg.create_pool(NoTls).unwrap();
    for i in 1..10 {
        let mut client = pool.get().await.unwrap();
        let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
        let rows = client.query(&stmt, &[&i]).await.unwrap();
        let value: i32 = rows[0].get(0);
        assert_eq!(value, i + 1);
    }
}
```

**Note:** The code above uses the crate name `config_crate` because of the
`config` feature and both features and dependencies share the same namespace.
In your own code you will probably want to use `::config::ConfigError` and
`::config::Config` instead.

## Example using an existing `tokio_postgres::Config` object

```rust,ignore
use std::env;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let mut pg_config = tokio_postgres::Config::new();
    pg_config.host_path("/run/postgresql");
    pg_config.host_path("/tmp");
    pg_config.user(env::var("USER").unwrap().as_str());
    pg_config.dbname("deadpool");
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast
    };
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
    let pool = Pool::new(mgr, 16);
    for i in 1..10 {
        let mut client = pool.get().await.unwrap();
        let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
        let rows = client.query(&stmt, &[&i]).await.unwrap();
        let value: i32 = rows[0].get(0);
        assert_eq!(value, i + 1);
    }
}
```

## FAQ

- **The database is unreachable. Why does the pool creation not fail?**

  Deadpool has [identical startup and runtime behaviour](https://crates.io/crates/deadpool/#reasons-for-yet-another-connection-pool)
  and therefore the pool creation will never fail.

  If you want your application to crash on startup if no database
  connection can be established just call `pool.get().await` right after
  creating the pool.

- **Why are connections retrieved from the pool sometimes unuseable?**

  In `deadpool-postgres 0.5.5` a new recycling method was implemented which
  is the default since `0.8`. With that recycling method the manager no
  longer performs a test query prior returning the connection but relies
  solely on `tokio_postgres::Client::is_closed` instead. Under some rare
  circumstances (e.g. unreliable networks) this can lead to `tokio_postgres`
  not noticing a disconnect and reporting the connection as useable.

  The old and slightly slower recycling method can be enabled by setting
  `ManagerConfig::recycling_method` to `RecyclingMethod::Verified` or when
  using the `config` crate by setting `PG__MANAGER__RECYCLING_METHOD=Verified`.

- **How can I enable features of the `tokio-postgres` crate?**

  Make sure that you depend on the same version of `tokio-postgres` as
  `deadpool-postgres` does and enable the needed features in your own
  `Crate.toml` file:

  ```toml
  [dependencies]
  deadpool-postgres = { version = "0.7" }
  tokio-postgres = { version = "0.7", features = ["with-uuid-0_8"] }
  ```

  **Important:** The version numbers of `deadpool-postgres` and
  `tokio-postgres` do not necessarily match. It is just a coincidence
  that both crates have the same MAJOR and MINOR version number at the
  time of this writing.

- **How can I clear the statement cache?**

  You can call `pool.manager().statement_cache.clear()` to clear all
  statement caches or `pool.manager().statement_cache.clear()` to remove
  a single statement from all caches.

  **Important:** The `ClientWrapper` also provides a `statement_cache`
  field which has `clear()` and `remove()` methods which only affect
  a single client.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
