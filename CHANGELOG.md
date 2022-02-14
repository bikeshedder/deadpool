# Change Log

## v0.9.3 (unreleased)

* Add `Pool::retain` method
* Fix `Pool::get_timeouts` method
* Deprecate `managed::Pool::try_get`
* Add `Pool::timeouts` method

## v0.9.2

* `PoolConfig` now implements `Serialize`

## v0.9.1

* Deprecate `managed::sync` module in favor of `deadpool-sync` crate
* Extract `runtime` module as separate `deadpool-runtime` crate

## v0.9.0

* __Breaking:__ Replace `config` feature with `serde` (opted out by default)
* Fix `std::error::Error::source` implementations for library errors
* Add `Runtime::spawn_blocking` method
* Add `Runtime::spawn_blocking_background` method
* Remove `Runtime::None` in favor of `Option<Runtime>`
* Remove `Pool::new` method
* Add `Pool::builder` method and `PoolBuilder` struct
* Add `Object::metrics` method and `Metrics` struct
* Update `tokio` dependency to version `1.5.0`
* Add `post_create`, `pre_recycle` and `post_recycle` hooks
* Add `Pool::resize` method
* Add `managed_reexports` macro

## v0.8.2

* Add `deadpool-diesel` to README
* Add `Sync + Send` as supertrait to `Manager`
* Fix usage of `PhantomData` in `Pool` struct: `Pool is now `Sync` regardless of the wrapper.

## v0.8.1

* Add `Object::pool` method

## v0.8.0

* Add support for closing pools
* Replace `crossbeam-queue` by `Mutex<VecDeque<_>>`
* Fix invalid `size` and `available` counts when recycling fails
* Update `config` dependency to version `0.11`
* Remove deprecated `from_env` methods
* Add support for wrappers returned by the pool
* Use associated types for traits

## v0.7.0

* Update `tokio` dependency to version `1`

## v0.6.0

* Update `tokio` dependency to version `0.3`
* Update `crossbeam-queue` dependency to version `0.3`
* Remove deprecated `deadpool::*` types
* Add `deadpool-memcached` to README

## v0.5.2

* Deprecate `managed::Config::from_env`
* Deprecate `unmanaged::Config::from_env`

## v0.5.1

* Add `managed::Object::take` method

## v0.5.0

* Move current pool implementation into `managed` module
* Add unmanaged version of the `Pool` which does not use a `Manager`
  to create and recycle objects.
* Add feature flags `"managed"` and `"unmanaged"` to enable only parts
  of this crate.
* Add `max_size` to pool `Status`
* Add support for `config` crate

## v0.4.3

* Add `std::error::Error` implementation for `PoolError` and `RecycleError`.
  This makes it more convenient to use the `?` operator.

## v0.4.2

* Replace `tokio::sync::mpsc::channel` by `crossbeam_queue::ArrayQueue`
  which gets rid of the mutex when fetching an object from the pool.

## v0.4.1

* Make `Pool::timeout_get` public

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
  aborted. This is related to <https://github.com/tokio-rs/tokio/issues/1898>
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
