#!/bin/bash
set -e

PACKAGE="$1"
echo "$PACKAGE"

psql -U lemmy -d postgres -c "DROP DATABASE lemmy;"
psql -U lemmy -d postgres -c "CREATE DATABASE lemmy;"

export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
# tests are executed in working directory crates/api (or similar),
# so to load the config we need to traverse to the repo root
export LEMMY_CONFIG_LOCATION=../../config/config.hjson
export RUST_BACKTRACE=1

if [ -n "$PACKAGE" ];
then
  cargo test -p $PACKAGE --all-features --no-fail-fast
else
  cargo test --workspace --all-features --no-fail-fast
fi

# Add this to do printlns: -- --nocapture
