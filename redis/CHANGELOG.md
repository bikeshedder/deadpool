# Change Log

## v0.7.1

* Change `redis` dependency to version range `0.19` to `0.20`

## v0.7.0

* Update `deadpool` dependency to version `0.3`
* Update `redis` dependency to version `0.19`
* Disable `redis` default features
* Mark `Config::from_env` as deprecated
* Re-export `deadpool::managed::PoolConfig`

## v0.6.1

* Change `redis` dependency to version range `0.15` to `0.17`

## v0.6.0

* Update `redis` dependency to version `0.16.0`
* Re-export `redis` crate

## v0.5.2

* Update `redis` dependency to version `0.15.1`
* Add `#[derive(Clone)]` to `Config` struct
* Add `Connection` type alias

## v0.5.1

* Disable `default-features` for `deadpool` dependency

## v0.5.0

* Update `redis` dependency to version `0.14.0`
* Rename `query` to `query_async` to match API of `redis` crate
* Rename `execute` to `execute_async` to match API of `redis` crate
* Add support for `config` crate

## v0.4.1

* Add `PoolError` type alias

## v0.4.0

* Rename `Connection` to `ConnectionWrapper`
* Add `Connection` type alias

## v0.3.0

* Add pipeline support
* Add wrappers for `Cmd` and `Pipeline` to mimick the API of the `redis` crate
* Make recycling more robust by changing the `Manager::recycle` to a non
  consuming API.
* Add proper connection using the `PING` command

## v0.2.0

* First release
