#!/usr/bin/env bash
set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd $CWD/../

PACKAGE="$1"
echo "$PACKAGE"

source scripts/start_dev_db.sh

# tests are executed in working directory crates/api (or similar),
# so to load the config we need to traverse to the repo root
export LEMMY_CONFIG_LOCATION=../../config/config.hjson
export RUST_BACKTRACE=1

cargo install cargo-llvm-cov

# Create lcov.info file, which is used by things like the Coverage Gutters extension for VS Code
cargo llvm-cov --workspace --all-features --no-fail-fast --lcov --output-path lcov.info

# Add this to do printlns: -- --nocapture

pg_ctl stop --silent
rm -rf $PGDATA
