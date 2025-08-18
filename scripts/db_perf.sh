#!/usr/bin/env bash

# This script runs crates/db_views/post/src/db_perf/mod.rs, which lets you see info related to database query performance, such as query plans.

set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd "$CWD/../"

source scripts/start_dev_db.sh

export LEMMY_CONFIG_LOCATION=$(pwd)/config/config.hjson
export RUST_BACKTRACE=1

cargo test -p lemmy_db_views_post --features full --no-fail-fast db_perf -- --nocapture

pg_ctl stop --silent

# $PGDATA directory is kept so log can be seen
