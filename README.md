# Deadpool [![Latest Version](https://img.shields.io/crates/v/deadpool.svg)](https://crates.io/crates/deadpool) [![Build Status](https://travis-ci.org/bikeshedder/deadpool.svg?branch=master)](https://travis-ci.org/bikeshedder/deadpool)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate provides two implementations:

- Managed pool (`deadpool::managed::Pool`)  
  - Creates and recycles objects as needed  
  - Useful for [database connection pools](#database-connection-pools)
  - Enabled via the `managed` feature in your `Cargo.toml`

- Unmanaged pool (`deadpool::unmanaged::Pool`)  
  - All objects either need to to be created by the user and added to the
    pool manually. It is also possible to create a pool from an existing
    collection of objects.
  - Enabled via the `unmanaged` feature in your `Cargo.toml`

## Features

| Feature | Description | Extra dependencies | Default |
| ------- | ----------- | ------------------ | ------- |
| `managed` | Enable managed pool implementation | – | yes |
| `unmanaged` | Enable unmanaged pool implementation | `async-trait` | yes |
| `config` | Enable support for [config](https://crates.io/crates/config) crate | `config`, `serde/derive` | yes |

## Managed pool (aka. connection pool)

This is the obvious choice for connection pools of any kind. Deadpool already
comes with a couple of [database connection pools](#database-connection-pools)
which work out of the box.

### Example

```rust
use async_trait::async_trait;

#[derive(Debug)]
enum Error { Fail }

struct Computer {}
struct Manager {}
type Pool = deadpool::managed::Pool<Computer, Error>;

impl Computer {
    async fn get_answer(&self) -> i32 {
        42
    }
}

#[async_trait]
impl deadpool::managed::Manager<Computer, Error> for Manager {
    async fn create(&self) -> Result<Computer, Error> {
        Ok(Computer {})
    }
    async fn recycle(&self, conn: &mut Computer) -> deadpool::managed::RecycleResult<Error> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let mgr = Manager {};
    let pool = Pool::new(mgr, 16);
    let mut conn = pool.get().await.unwrap();
    let answer = conn.get_answer().await;
    assert_eq!(answer, 42);
}
```

### Database connection pools

Deadpool supports various database backends by implementing the
`deadpool::managed::Manager` trait. The following backends are
currently supported:

Backend                                                     | Crate
----------------------------------------------------------- | -----
[tokio-postgres](https://crates.io/crates/tokio-postgres)   | [deadpool-postgres](https://crates.io/crates/deadpool-postgres)
[lapin](https://crates.io/crates/lapin) (AMQP)              | [deadpool-lapin](https://crates.io/crates/deadpool-lapin)
[redis](https://crates.io/crates/redis)                     | [deadpool-redis](https://crates.io/crates/deadpool-redis)

### Reasons for yet another connection pool

Deadpool is by no means the only pool implementation available. It does
things a little different and that is the main reason for it to exist:

- **Deadpool is compatible with any executor.** Objects are returned to the
  pool using the `Drop` trait. The health of those objects is checked upon
  next retrieval and not when they are returned. Deadpool never performs any
  actions in the background. This is the reason why deadpool does not need
  to spawn futures and does not rely on a background thread or task of any
  type.

- **Identical startup and runtime behaviour**. When writing long running
  application there usually should be no difference between startup and
  runtime if a database connection is temporarily not available. Nobody
  would expect an application to crash if the database becomes unavailable
  at runtime. So it should not crash on startup either. Creating the pool
  never fails and errors are only ever returned when calling `Pool::get()`.

  If you really want your application to crash on startup if objects can
  not be created on startup simply call
  `pool.get().await.expect("DB connection failed")` right after creating
  the pool.

- **Deadpool is fast.** The code which returns connections to the pool
  contains no blocking code and retrival uses only one locking primitive.

- **Deadpool is simple.** Dead simple. There is very little API surface.
  The actual code is barely 100 lines of code and lives in the two functions
  `Pool::get` and `Object::drop`.

### Differences to other connection pool implementations

- [`r2d2`](https://crates.io/crates/r2d2) provides a lot more configuration
  options but only provides a synchroneous interface.

- [`bb8`](https://crates.io/crates/bb8) uses a callback based interface (See
  [`pool.run`](https://docs.rs/bb8/0.3.1/bb8/struct.Pool.html#method.run))
  and provides the same configuration options as `r2d2`. At the time of
  writing there is no official release which supports `async/.await`.

- [`mobc`](https://crates.io/crates/mobc) provides an `async/.await` based
  interface and provides a lot more configuration options. It requires an
  executor though and the code is a lot more complex.

## Unmanaged pool

An unmanaged pool is useful when you can't write a manager for the objects
you want to pool or simply don't want to. This pool implementation is slightly
faster than the managed pool because it does not use a `Manager` trait to
`create` and `recycle` objects but leaves it up to the user.

### Unmanaged pool example

```rust
use deadpool::unmanaged::Pool;

struct Computer {}

impl Computer {
    async fn get_answer(&self) -> i32 {
        42
    }
}

#[tokio::main]
async fn main() {
    let pool = Pool::from(vec![
        Computer {},
        Computer {},
    ]);
    let s = pool.get().await;
    assert_eq!(s.get_answer().await, 42);
}
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT)>

at your option.
