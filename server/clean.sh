#!/bin/sh

cargo update
cargo fmt
cargo check
cargo clippy
