#!/bin/sh
set -e

export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
# Commenting since this will overwrite schema.rs, which will break things now
# diesel migration run
# Integration tests only work on stable due to a bug in config-rs
# https://github.com/mehcode/config-rs/issues/158
RUST_BACKTRACE=1 RUST_TEST_THREADS=1 \
  cargo +1.47.0 test --workspace --no-fail-fast
