#!/usr/bin/env bash
set -e

# This check is only used for CI.

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd "$CWD/../"

# Copy the files to a temp dir
TMP_DIR=$(mktemp -d)
cp -a migrations/. $TMP_DIR/migrations
cp -a crates/diesel_utils/replaceable_schema/. $TMP_DIR/replaceable_schema

# Format the new files
find $TMP_DIR -type f -name '*.sql' -print0 | xargs -0 -P 10 -L 10 pg_format -i

# Diff the directories
diff -r migrations $TMP_DIR/migrations
diff -r crates/diesel_utils/replaceable_schema $TMP_DIR/replaceable_schema
