#!/usr/bin/env bash

# This script runs crates/lemmy_db_perf/src/main.rs, which lets you see info related to database query performance, such as query plans.

set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd $CWD/../

source scripts/start_dev_db.sh

export LEMMY_CONFIG_LOCATION=config/config.hjson
export RUST_BACKTRACE=1

cargo run --package lemmy_db_perf -- "$@"

pg_ctl stop --silent

# $PGDATA directory is kept so log can be seen
