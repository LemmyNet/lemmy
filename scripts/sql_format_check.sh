#!/usr/bin/env bash
set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd $CWD/../

git ls-files '**/*.sql' | while read FILE
do
  TMP_FILE="/tmp/tmp_pg_format.sql"
  pg_format $FILE > $TMP_FILE
  diff $FILE $TMP_FILE
done
