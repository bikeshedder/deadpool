# Deadpool [![Latest Version](https://img.shields.io/crates/v/deadpool.svg)](https://crates.io/crates/deadpool) [![Build Status](https://travis-ci.org/bikeshedder/deadpool.svg?branch=master)](https://travis-ci.org/bikeshedder/deadpool)

Deadpool is a dead simple async pool for connections and objects
of any type.

## Backends

Deadpool supports various backends by implementing the `deadpool::Manager`
trait. The following backends are currently supported:

Backend                                                     | Crate
----------------------------------------------------------- | -----
[tokio-postgres](https://crates.io/crates/tokio-postgres)   | [deadpool-postgres](https://crates.io/crates/deadpool-postgres)
[lapin](https://crates.io/crates/lapin) (AMQP)              | [deadpool-lapin](https://crates.io/crates/deadpool-lapin)
[redis](https://crates.io/crates/redis)                     | [deadpool-redis](https://crates.io/crates/deadpool-redis)

## Example

```rust
use async_trait::async_trait;

#[derive(Debug)]
enum Error { Fail }

struct Connection {}

type Pool = deadpool::Pool<Connection, Error>;

impl Connection {
    async fn new() -> Result<Self, Error> {
        Ok(Connection {})
    }
    async fn check_health(&self) -> bool {
        true
    }
    async fn do_something(&self) -> String {
        "Horray!".to_string()
    }
}

struct Manager {}

#[async_trait]
impl deadpool::Manager<Connection, Error> for Manager
{
    async fn create(&self) -> Result<Connection, Error> {
        Connection::new().await
    }
    async fn recycle(&self, conn: &mut Connection) -> Result<(), Error> {
        if conn.check_health().await {
            Ok(())
        } else {
            Err(Error::Fail)
        }
    }
}

#[tokio::main]
async fn main() {
    let mgr = Manager {};
    let pool = Pool::new(mgr, 16);
    let mut conn = pool.get().await.unwrap();
    let value = conn.do_something().await;
    assert_eq!(value, "Horray!".to_string());
}
```

For a more complete example please see
[`deadpool-postgres`](https://crates.io/crates/deadpool-postgres)

## Reasons for yet another pool implementation

Deadpool is by no means the only pool implementation available. It does
things a little different and that is the reason for it to exist:

* **Deadpool is compatible with any executor.** Objects are returned to the
  pool using the `Drop` trait. The health of those objects is checked upon
  next retrieval and not when they are returned. Deadpool never performs any
  action in the background. This is the reason why deadpool does not need
  to spawn futures and does not rely on a background thread or task of any
  type.

* **Identical startup and runtime behaviour**. When writing long running
  application there usually should be no difference between startup and
  runtime if a database connection is temporarily not available. Nobody
  would expect an application to crash if the database becomes unavailable
  at runtime. So it should not crash on startup either. Creating the pool
  never fails and errors are only ever returned when calling `Pool::get()`.

  If you really want your application to crash on startup if objects can
  not be created on startup simply call
  `pool.get().expect("DB connection failed")` right after creating the pool.

* **Deadpool is fast.** The code which returns connections to the pool
  contains no blocking code and retrival uses only one locking primitive.
  Everything else is implemented using non-locking atomic counters.

* **Deadpool is simple.** Dead simple. There is very little API surface and
  the actual code is barely 100 lines of code.

## Differences to other pool implementations

* [`r2d2`](https://crates.io/crates/r2d2) only provides a synchroneous
  interface. It also is more complex and needs a lot more code.

* [`bb8`](https://crates.io/crates/bb8) uses a callback based interface: See
  [`pool.run`](https://docs.rs/bb8/0.3.1/bb8/struct.Pool.html#method.run)

* [`mobc`](https://crates.io/crates/mobc) provides an `async/.await` based
  interface and provides a lot more configuration options. The downside
  of this being added code complexity and the need for an executor which
  `deadpool` does not need.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
