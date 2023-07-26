#!/usr/bin/env bash
set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd $CWD/../

CMD_ARGS=
TEST_ARGS=

PACKAGE="$1"
if [ -n "$PACKAGE" ];
then
  CMD_ARGS="-p $PACKAGE --all-features"
  echo "$PACKAGE"
else
  CMD_ARGS="--workspace"
fi

TEST="$2"
if [ -n "$TEST" ];
then
  TEST_ARGS="-- $TEST"
  echo $TEST
fi

CMD="cargo test $CMD_ARGS --no-fail-fast $TEST_ARGS"
echo Running: $CMD

source scripts/start_dev_db.sh

# tests are executed in working directory crates/api (or similar),
# so to load the config we need to traverse to the repo root
export LEMMY_CONFIG_LOCATION=../../config/config.hjson
export RUST_BACKTRACE=1

$CMD

# Add this to do printlns: -- --nocapture

pg_ctl stop
rm -rf $PGDATA
