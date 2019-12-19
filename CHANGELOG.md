# Change Log

## v0.4.0

* Add support for timeouts
* Make fields of pool status public
* Fix possible deadlock and make implementation a lot simpler by using
  the new tokio `Semaphore` and `Receiver::try_recv`.
* Add `Pool::try_get` and `Pool::timeout_get` functions

## v0.3.0

* Add `deadpool-lapin` to README
* Add `deadpool-redis` to README
* Fix possible stale state and deadlock if a future calling `Pool::get` is
  aborted. This is related to https://github.com/tokio-rs/tokio/issues/1898
* Make recycling more robust by changing the `Manager::recycle` to a non
  consuming API.

## v0.2.3

* Add documentation for `docs.rs`
* Remove `PoolInner` and `PoolSize` struct from public interface
* Improve example in `README.md` and crate root

## v0.2.2

* Update to `tokio 0.2`

## v0.2.1

* Version skipped; only `tokio-postgres` was updated.

## v0.2.0

* Split `deadpool` and `deadpool-postgres` in separate crates instead of
    one with feature flags.

## v0.1.0

* First release
