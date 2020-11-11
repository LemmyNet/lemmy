#!/bin/sh
export DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
diesel migration run
export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
# Integration tests only work on stable due to a bug in config-rs
# https://github.com/mehcode/config-rs/issues/158
RUST_BACKTRACE=1 RUST_TEST_THREADS=1 \
  cargo +stable test --workspace --no-fail-fast
