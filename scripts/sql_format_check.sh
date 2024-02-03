#!/usr/bin/env bash
set -e

# This check is only used for CI.

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd $CWD/../

# Copy the files to a temp dir
TMP_DIR=$(mktemp -d)
cp -a migrations/. $TMP_DIR/migrations
cp -a crates/db_schema/src/schema_setup/. $TMP_DIR/schema_setup

# Format the new files
find $TMP_DIR -type f -name '*.sql' -exec pg_format -i {} +

# Diff the directories
diff -r migrations $TMP_DIR/migrations
diff -r crates/db_schema/src/schema_setup $TMP_DIR/schema_setup
