#!/bin/sh
set -e

psql -U lemmy -d postgres -c "DROP DATABASE lemmy;"
psql -U lemmy -d postgres -c "CREATE DATABASE lemmy;"

export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
RUST_BACKTRACE=1 \
  cargo test --workspace --no-fail-fast
