#!/bin/sh
set -e

psql -U lemmy -d postgres -c "DROP DATABASE lemmy;"
psql -U lemmy -d postgres -c "CREATE DATABASE lemmy;"

export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
# Commenting since this will overwrite schema.rs, which will break things now
# diesel migration run
# Integration tests only work on stable due to a bug in config-rs
# https://github.com/mehcode/config-rs/issues/158
RUST_BACKTRACE=1 RUST_TEST_THREADS=1 \
  cargo test --workspace --no-fail-fast
