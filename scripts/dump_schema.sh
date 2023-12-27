#!/usr/bin/env bash
set -e

# Dumps database schema, not including things added by replaceable_schema.sql

source CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd $CWD/../

source scripts/start_dev_db.sh

diesel migration run
pg_dump --no-owner --no-privileges --no-table-access-method --schema-only --no-sync -f schema.sqldump

pg_ctl stop
rm -rf $PGDATA
