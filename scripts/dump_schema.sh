#!/usr/bin/env bash
set -e

# Dumps database schema, not including things that are added outside of migrations

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd $CWD/../

source scripts/start_dev_db.sh

cargo run --package lemmy_server -- migration run
pg_dump --no-owner --no-privileges --no-table-access-method --schema-only --exclude-schema=r --no-sync -f schema.sqldump

pg_ctl stop
rm -rf $PGDATA
