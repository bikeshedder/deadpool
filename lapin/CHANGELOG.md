# Change Log

## v0.11.0

- Update `deadpool` dependency to version `0.10`

## v0.10.0

* Update `lapin` dependency to version `2.0`

## v0.9.1

* Config structs now implement `Serialize`

## v0.9.0

* __Breaking:__ Replace `config` feature with `serde` (opted out by default)
* Remove unused `futures` and `log` dependencies
* Update `deadpool` to `0.9`

## v0.8.0

* Update `config` dependency to version `0.11`
* Remove deprecated `from_env` methods
* Add `rt_tokio_1` and `rt_async-std_1` features

## v0.7.0

* Update `tokio` dependency to version `1`
* Update `tokio-amqp` dependency to version `1`
* Mark `Config::from_env` as deprecated
* Re-export `deadpool::managed::PoolConfig`

## v0.6.2

* Add support for `ConnectionProperties`

## v0.6.1

* Add support for `deadpool 0.5` (`tokio 0.2`) and `deadpool 0.6` (`tokio 0.3`)

## v0.6.0

* Update `lapin` dependency to version 1.0.0
* Re-export for `lapin` crate

## v0.5.0

* First release
