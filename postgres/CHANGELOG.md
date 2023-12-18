# Change Log

## v0.12.1

- Fix `Config::url` handling

## v0.12.0

- Add `load_balance_hosts` to `Config` struct.
- Add `hostaddr` and `hostaddrs` to `Config` struct.
- Add `url` field to `Config` struct. This enables parsing of
  connection URLs.

## v0.11.0

- **BREAKING:** Disconnect immediately from the database when dropping
  clients. This is considered a breaking change as in previous versions
  the connections would stick around until all queued queries were
  processed. This was considered a potential leak of resources. The
  `Connection` object is now tied to the lifetime of the `ClientWrapper`
  and dropped as soon as possible. The disconnect is not graceful and
  you might see error messages in the database log.
- Update `deadpool` dependency to version `0.10`
- Bump up MSRV to `1.63` to match the one of `tokio`

## v0.10.5

- Fix infinite recursion in `GenericClient`

## v0.10.4 (yanked)

- Add `GenericClient`

## v0.10.3

* Make `Transaction::statement_cache` field public

## v0.10.2

* Export `TargetSessionAttrs` and `ChannelBinding` enums (part of `Config`
  struct)

## v0.10.1

* Config structs now implement `Serialize`

## v0.10.0

* __Breaking:__ Replace `config` feature with `serde` (opted out by default)
* Re-export `deadpool::managed::Timeouts`
* Add `Runtime` parameter to `Config::create_pool` method
* Remove redundant `futures` dependency

## v0.9.0

* Remove generic `<T>` parameter from `Manager`, `Pool` and `Client`
  types. This parameter was added by accident when deadpool switched
  to using associated types.

## v0.8.1

* Update `tokio-postgres` dependency to version `0.7.2`
  This crate depends on the `GenericClient::client` method
  which was added in `tokio-postgres` version `0.7.2`.

## v0.8.0

* Do not detect unix domain socket paths at config creation.
* Update `config` dependency to version `0.11`
* Remove deprecated `from_env` methods
* Add `Manager::statement_caches` field which provides access
  to managing the statement cache for all clients.
* Rename `prepare` to `prepare_cached` and `prepare_typed` to
  `prepare_typed_cached`. This makes the non-caching prepare
  methods available without having to dereference `ClientWrapper`
  or `Transaction` objects first.
* Add `rt_tokio_1` and `rt_async-std_1` features
* Enable `RecyclingMethod::Fast` by default

## v0.7.0

* Update `tokio` dependency to version `1`
* Update `tokio-postgres` dependency to version `0.7`
* Re-export `deadpool::managed::PoolConfig`
* Add `StatementCache::remove` method

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
