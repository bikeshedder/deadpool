# Deadpool

Deadpool is a dead simple async pool for connections and objects
of any type.

## Unique selling points

Objects are returned to the pool using the `Drop` trait. The health of
those objects it checked upon next retrieval and not when they are
returned. Deadpool never performs any action in the background. This
is the reason why deadpool is **compatible with any executor** of
futures and does not use a background thread or task of any type.

The behaviour of deadpool is identical during startup and runtime. Therefore
creating the pool never fails and errors are only ever returned when calling
`Pool::get`.

Deadpool is simple. Dead simple. There is very little API surface and the
actual code is barely 100 LoC.

## Backends

Backend                                                     | Crate
----------------------------------------------------------- | -----
[tokio-postgres](https://crates.io/crates/tokio-postrges)   | [deadpool-postgres](https://crates.io/crates/deadpool-postgres)

## Example

```rust
use async_trait::async_trait;
use deadpool_postgres::{Manager, Pool};

struct Error {}

struct Connection {}

impl Connection {
    async fn new() -> Result<Self, Error> {
        unimplemented!();
    }
    async fn check_health(&self) -> bool {
        unimplemented!();
    }
}

struct Manager {}

#[async_trait]
impl deadpool::Manager<Client, Error> for Manager
{
    async fn create(&self) -> Result<Connection, Error> {
        Connection::new().await
    }
    async fn recycle(&self, conn: Connection) -> Result<Client, Error> {
        if conn.check_health().await {
            conn
        } else {
            Connection::new().await
        }
    }
}

#[tokio::main]
fn main() {
    let mgr = Manager::new();
    let pool = Pool::new(mgr, 16);
    loop {
        let mut conn = pool.get().await.unwrap();
        let value = conn.do_something()
        println!("{}", value);
    }
}
```

For a more complete example please see
[`deadpool-postgres`](https://crates.io/crates/deadpool-postgres)

## How does Deadpool compare to other pool implementations?

- [`r2d2`](https://crates.io/crates/r2d2) only provides a synchroneous interface

- [`bb8`](https://crates.io/crates/bb8) uses a callback based interface:
  [https://docs.rs/bb8/0.3.1/bb8/struct.Pool.html#method.run](`pool.run`)

- [`mobc`](https://crates.io/crates/mobc) provides an `async/.await` based
  interface and provides a lot more configuration options. The downside
  of this being added code complexity and the need for an executor which
  `deadpool` does not need.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
