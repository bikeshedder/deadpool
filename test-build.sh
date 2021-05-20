#!/bin/bash

set -xe

cargo build
cargo build --no-default-features
cargo build --no-default-features --features managed
cargo build --no-default-features --features unmanaged
cargo build --no-default-features --features config
cargo build --no-default-features --features managed,config
cargo build --no-default-features --features unmanaged,config
cargo build --no-default-features --features managed,unmanaged

(
	cd postgres
	cargo build
	cargo build --no-default-features
	cargo build --no-default-features --features rt_tokio_1
	cargo build --no-default-features --features rt_async-std_1
)

(
	cd redis
	cargo build
	cargo build --no-default-features --features rt_tokio_1
	cargo build --no-default-features --features rt_async-std_1
)

(
	cd lapin
	cargo build
	cargo build --no-default-features
	cargo build --no-default-features --features rt_tokio_1
	cargo build --no-default-features --features rt_async-std_1
)

