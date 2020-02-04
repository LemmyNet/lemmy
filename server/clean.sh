#!/bin/sh

cargo update
cargo fmt
cargo check
cargo clippy
cargo outdated -R
