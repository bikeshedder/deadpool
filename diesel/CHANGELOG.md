# Change Log

## v0.3.1
* Expose the reexports from deadpool and deadpool_sync for mysql and postgres

## v0.3.0

* __Breaking:__ Replace `deadpool::managed::sync` by
  `deadpool-sync::SyncWrapper` which fixes the return type
  of the `interact` method.

## v0.2.0

* __Breaking:__ Replace `config` feature with `serde` (opted out by default)
* Fix `std::error::Error::source` implementations for library errors
* Remove unused `tokio` dependency
* Async unaware diesel connections are now wrapped inside
  a `deadpool::managed::sync::SyncWrapper` which ensures that
  all database operations are run in a separate threads.

## v0.1.2

* Remove `unsafe impl` by better usage of `PhantomData`

## v0.1.1

* Fix title and crates.io badge in README

## v0.1.0

* First release
