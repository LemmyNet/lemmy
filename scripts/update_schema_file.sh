#!/usr/bin/env bash
set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd "$CWD/../"

source scripts/start_dev_db.sh

cargo run --package lemmy_diesel_utils
diesel print-schema >crates/db_schema_file/src/schema.rs
cargo +nightly fmt --package lemmy_db_schema_file

pg_ctl stop
rm -rf $PGDATA
