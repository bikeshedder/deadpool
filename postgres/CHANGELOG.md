# Change Log

## v0.6.1 (unreleased)

* Re-export `deadpool::managed::PoolConfig`

## v0.6.0

* Update `tokio` dependency to version `0.3`
* Update `tokio-postgres` dependency to version `0.6`

## v0.5.6

* Add `Config::new` method
* Add `Client::build_transaction` method which makes it possible to
  use the `TransactionBuilder` with the statement cache.
* Add `RecyclingMethod::Clean` which works similar to `DISCARD ALL`
  but makes sure the statement cache is not rendered ineffective.
* Add `RecyclingMethod::Custom` which allows to execute arbitary SQL
  when recycling connections.
* Re-export `tokio_postgres` crate

## v0.5.5

* Deprecate `Config::from_env`
* Add `Manager::from_config`, `ManagerConfig` and `RecyclingMethod` which
  makes it possible to specify how connections are recycled. The current
  default recycling method is `Verified` which is the same as before. The
  upcoming `0.6` release of this crate will change the default to `Fast`.

## v0.5.4

* Implement `DerefMut` for `Transaction` wrapper
* Add `transaction` method to `Transaction` wrapper

## v0.5.3

* Add `#[derive(Clone)]` to `Config` struct
* Make `config` module public

## v0.5.2

* Disable `default-features` for `deadpool` dependency

## v0.5.1

* Fix windows support

## v0.5.0

* Add support for `config` crate

## v0.4.3

* `prepare` and `prepare_typed` now accept a `&self` instead of `&mut self`
  which fixes support for pipelining.

## v0.4.2

* Add `PoolError` type alias

## v0.4.1

* Update to `tokio-postgres 0.5.1`
* Add back `DerefMut` implementation for `deadpool_postgres::Client` which
  makes it compatible with code expecting `&mut tokio_postgres::Client`.
* Add statement cache support for `Client::prepare_typed` and
  `Transaction::prepare_typed`.

## v0.4.0

* Rename `Client` struct to `ClientWrapper`
* Add `Client` type alias

## v0.3.0

* Add `StatementCache` struct with the functions `size` and `clear` which
  are now accessible via `Connection::statement_cache` and
  `Transaction::statement_cache`.
* Make recycling more robust by changing the `Manager::recycle` to a non
  consuming API.

## v0.2.3

* Add documentation for `docs.rs`
* Improve example in `README.md` and crate root
* Fix `Transaction::commit` and `Transaction::rollback`

## v0.2.2

* Update to `tokio 0.2` and `tokio-postgres 0.5.0-alpha.2`

## v0.2.1

* `deadpool_postgres::Client` no longer implements `DerefMut` which was not
    needed anyways.
* `deadpool_postgres::Client.transaction` now returns a wrapped transaction
    object which utilizes the statement cache of the wrapped client.

## v0.2.0

* First release
