# Change Log

## v0.13.0 (unreleased)

* Update `deadpool` dependency to version `0.10`

## v0.12.0

- Update `redis` dependency to version `0.23`

## v0.11.1

- Export missing config structures:
  - `ConnectionAddr`
  - `ConnectionInfo`
  - `RedisConnectionInfo`
- Add `Config::from_connection_info` method

## v0.11.0

- Update `redis` dependency to version `0.22`

## v0.10.2

- Fix bug causing connections not to be recycled

## v0.10.1

- Config structs now implement `Serialize`

## v0.10.0

- **Breaking:** Replace `config` feature with `serde` (opted out by default)
- Fix `std::error::Error::source` implementations for library errors
- Remove unused `log` dependency

## v0.9.0

- Remove `deadpool_redis::Connection` type alias
- Rename `deadpool_redis::ConnectionWrapper` to `Connection`
- Update `redis` dependency to version `0.21`
- Add `Config::from_url` method

## v0.8.1

- Export `PoolCreateError`

## v0.8.0

- Update `config` dependency to version `0.11`
- Remove deprecated `from_env` methods
- Remove wrappers for `Cdm` and `Pipe`. The pool now returns a
  `ConnectionWrapper` rather than an `Object<ConnectionWrapper>` which
  implements the `redis::aio::ConnectionLike` trait and therefore can
  be used with plain `Cmd` and `Pipe` objects from the `redis` crate.
- Add support for new `redis::ConnectionInfo` structure.
- Change `redis` dependency to version `0.20`
- Make connection recycling more robust by checking the PING
  response. This works around `Cmd::query_async` not being drop
  safe in `redis` version `0.10` and earlier.
- Add `rt_tokio_1` and `rt_async-std_1` features

## v0.7.1

- Change `redis` dependency to version range `0.19` to `0.20`

## v0.7.0

- Update `deadpool` dependency to version `0.3`
- Update `redis` dependency to version `0.19`
- Disable `redis` default features
- Mark `Config::from_env` as deprecated
- Re-export `deadpool::managed::PoolConfig`

## v0.6.1

- Change `redis` dependency to version range `0.15` to `0.17`

## v0.6.0

- Update `redis` dependency to version `0.16.0`
- Re-export `redis` crate

## v0.5.2

- Update `redis` dependency to version `0.15.1`
- Add `#[derive(Clone)]` to `Config` struct
- Add `Connection` type alias

## v0.5.1

- Disable `default-features` for `deadpool` dependency

## v0.5.0

- Update `redis` dependency to version `0.14.0`
- Rename `query` to `query_async` to match API of `redis` crate
- Rename `execute` to `execute_async` to match API of `redis` crate
- Add support for `config` crate

## v0.4.1

- Add `PoolError` type alias

## v0.4.0

- Rename `Connection` to `ConnectionWrapper`
- Add `Connection` type alias

## v0.3.0

- Add pipeline support
- Add wrappers for `Cmd` and `Pipeline` to mimick the API of the `redis` crate
- Make recycling more robust by changing the `Manager::recycle` to a non
  consuming API.
- Add proper connection using the `PING` command

## v0.2.0

- First release
