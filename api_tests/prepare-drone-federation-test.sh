#!/usr/bin/env bash
# IMPORTANT NOTE: this script does not use the normal LEMMY_DATABASE_URL format
#   it is expected that this script is called by run-federation-test.sh script.
set -e

if [ -z "$LEMMY_LOG_LEVEL" ];
then
  LEMMY_LOG_LEVEL=info
fi

export RUST_BACKTRACE=1
export RUST_LOG="warn,lemmy_server=$LEMMY_LOG_LEVEL,lemmy_federate=$LEMMY_LOG_LEVEL,lemmy_api=$LEMMY_LOG_LEVEL,lemmy_api_common=$LEMMY_LOG_LEVEL,lemmy_api_crud=$LEMMY_LOG_LEVEL,lemmy_apub=$LEMMY_LOG_LEVEL,lemmy_db_schema=$LEMMY_LOG_LEVEL,lemmy_db_views=$LEMMY_LOG_LEVEL,lemmy_routes=$LEMMY_LOG_LEVEL,lemmy_utils=$LEMMY_LOG_LEVEL,lemmy_websocket=$LEMMY_LOG_LEVEL"

export LEMMY_TEST_FAST_FEDERATION=1 # by default, the persistent federation queue has delays in the scale of 30s-5min

PICTRS_PATH="api_tests/pict-rs"
PICTRS_EXPECTED_HASH="7f7ac2a45ef9b13403ee139b7512135be6b060ff2f6460e0c800e18e1b49d2fd  api_tests/pict-rs"

# Pictrs setup. Download file with hash check and up to 3 retries. 
if [ ! -f "$PICTRS_PATH" ]; then
  count=0
  while [ ! -f "$PICTRS_PATH" ] && [ "$count" -lt 3 ]
  do
    # This one sometimes goes down
    curl "https://git.asonix.dog/asonix/pict-rs/releases/download/v0.5.17-pre.9/pict-rs-linux-amd64" -o "$PICTRS_PATH"
    # curl "https://codeberg.org/asonix/pict-rs/releases/download/v0.5.5/pict-rs-linux-amd64" -o "$PICTRS_PATH"
    PICTRS_HASH=$(sha256sum "$PICTRS_PATH")
    if [[ "$PICTRS_HASH" != "$PICTRS_EXPECTED_HASH" ]]; then
      echo "Pictrs binary hash mismatch, was $PICTRS_HASH but expected $PICTRS_EXPECTED_HASH"
      rm "$PICTRS_PATH"
      let count=count+1
    fi
  done
  chmod +x "$PICTRS_PATH"
fi

./api_tests/pict-rs \
  run -a 0.0.0.0:8080 \
  --danger-dummy-mode \
  --api-key "my-pictrs-key" \
  filesystem -p /tmp/pictrs/files \
  sled -p /tmp/pictrs/sled-repo 2>&1 &

for INSTANCE in lemmy_alpha lemmy_beta lemmy_gamma lemmy_delta lemmy_epsilon; do
  echo "DB URL: ${LEMMY_DATABASE_URL} INSTANCE: $INSTANCE"
  psql "${LEMMY_DATABASE_URL}/lemmy" -c "DROP DATABASE IF EXISTS $INSTANCE"
  echo "create database"
  psql "${LEMMY_DATABASE_URL}/lemmy" -c "CREATE DATABASE $INSTANCE"
done

if [ -z "$DO_WRITE_HOSTS_FILE" ]; then
  if ! grep -q lemmy-alpha /etc/hosts; then
    echo "Please add the following to your /etc/hosts file, then press enter:

      127.0.0.1       lemmy-alpha
      127.0.0.1       lemmy-beta
      127.0.0.1       lemmy-gamma
      127.0.0.1       lemmy-delta
      127.0.0.1       lemmy-epsilon"
    read -p ""
  fi
else
  for INSTANCE in lemmy-alpha lemmy-beta lemmy-gamma lemmy-delta lemmy-epsilon; do
    echo "127.0.0.1 $INSTANCE" >>/etc/hosts
  done
fi

echo "$PWD"

LOG_DIR=target/log
mkdir -p $LOG_DIR

echo "start alpha"
LEMMY_CONFIG_LOCATION=./docker/federation/lemmy_alpha.hjson \
  LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_alpha" \
  target/lemmy_server >$LOG_DIR/lemmy_alpha.out 2>&1 &

echo "start beta"
LEMMY_CONFIG_LOCATION=./docker/federation/lemmy_beta.hjson \
  LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_beta" \
  target/lemmy_server >$LOG_DIR/lemmy_beta.out 2>&1 &

echo "start gamma"
LEMMY_CONFIG_LOCATION=./docker/federation/lemmy_gamma.hjson \
  LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_gamma" \
  target/lemmy_server >$LOG_DIR/lemmy_gamma.out 2>&1 &

echo "start delta"
LEMMY_CONFIG_LOCATION=./docker/federation/lemmy_delta.hjson \
  LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_delta" \
  target/lemmy_server >$LOG_DIR/lemmy_delta.out 2>&1 &

echo "start epsilon"
LEMMY_CONFIG_LOCATION=./docker/federation/lemmy_epsilon.hjson \
  LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_epsilon" \
  target/lemmy_server >$LOG_DIR/lemmy_epsilon.out 2>&1 &

echo "wait for all instances to start"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'lemmy-alpha:8541/api/v4/site')" != "200" ]]; do sleep 1; done
echo "alpha started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'lemmy-beta:8551/api/v4/site')" != "200" ]]; do sleep 1; done
echo "beta started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'lemmy-gamma:8561/api/v4/site')" != "200" ]]; do sleep 1; done
echo "gamma started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'lemmy-delta:8571/api/v4/site')" != "200" ]]; do sleep 1; done
echo "delta started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'lemmy-epsilon:8581/api/v4/site')" != "200" ]]; do sleep 1; done
echo "epsilon started. All started"
