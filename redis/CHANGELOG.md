# Change Log

## v0.3.0 (unreleased)

* Add pipeline support
* Add wrappers for `Cmd` and `Pipeline` to mimick the API of the `redis` crate
* Make recycling more robust by changing the `Manager::recycle` to a non
  consuming API.
* Add proper connection using the `PING` command

## v0.2.0

* First release
