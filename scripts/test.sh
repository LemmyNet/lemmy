#!/usr/bin/env bash
set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd "$CWD/../"

PACKAGE="$1"
TEST="$2"

source scripts/start_dev_db.sh

# tests are executed in working directory crates/api (or similar),
# so to load the config we need to traverse to the repo root
export LEMMY_CONFIG_LOCATION=../../config/config.hjson
export RUST_BACKTRACE=1
export LEMMY_TEST_FAST_FEDERATION=1 # by default, the persistent federation queue has delays in the scale of 30s-5min

if [ -n "$PACKAGE" ];
then
  cargo test -p $PACKAGE --all-features --no-fail-fast $TEST
else
  cargo test --workspace --no-fail-fast
  # Testing lemmy utils all features in particular (for ts-rs bindings)
  cargo test -p lemmy_utils --all-features --no-fail-fast
fi

# Add this to do printlns: -- --nocapture

pg_ctl stop --silent
rm -rf $PGDATA
