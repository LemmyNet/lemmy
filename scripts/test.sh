#!/bin/sh
set -e

export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
export DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy

read -p "Clear database? " -n 1 -r
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  psql -U lemmy -d postgres -c "DROP DATABASE lemmy;"
  psql -U lemmy -d postgres -c "CREATE DATABASE lemmy;"
fi


# Integration tests only work on stable due to a bug in config-rs
# https://github.com/mehcode/config-rs/issues/158
RUST_BACKTRACE=1 RUST_TEST_THREADS=1 \
  cargo +1.47.0 test --workspace --no-fail-fast
