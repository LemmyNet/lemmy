#!/usr/bin/env bash
# IMPORTANT NOTE: this script does not use the normal LEMMY_DATABASE_URL format
#   it is expected that this script is called by run-federation-test.sh script.
set -e

export RUST_BACKTRACE=1
export RUST_LOG="warn,lemmy_server=debug,lemmy_api=debug,lemmy_api_common=debug,lemmy_api_crud=debug,lemmy_apub=debug,lemmy_db_schema=debug,lemmy_db_views=debug,lemmy_db_views_actor=debug,lemmy_db_views_moderator=debug,lemmy_routes=debug,lemmy_utils=debug,lemmy_websocket=debug"

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
    echo "127.0.0.1 $INSTANCE" >> /etc/hosts
  done
fi

echo "killall existing lemmy_server processes"
killall lemmy_server || true

echo "$PWD"

echo "start alpha"
LEMMY_CONFIG_LOCATION=./docker/federation/lemmy_alpha.hjson \
LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_alpha" \
target/lemmy_server >/tmp/lemmy_alpha.out 2>&1 &

echo "start beta"
LEMMY_CONFIG_LOCATION=./docker/federation/lemmy_beta.hjson \
LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_beta" \
target/lemmy_server >/tmp/lemmy_beta.out 2>&1 &

echo "start gamma"
LEMMY_CONFIG_LOCATION=./docker/federation/lemmy_gamma.hjson \
LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_gamma" \
target/lemmy_server >/tmp/lemmy_gamma.out 2>&1 &

echo "start delta"
# An instance with only an allowlist for beta
LEMMY_CONFIG_LOCATION=./docker/federation/lemmy_delta.hjson \
LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_delta" \
target/lemmy_server >/tmp/lemmy_delta.out 2>&1 &

echo "start epsilon"
# An instance who has a blocklist, with lemmy-alpha blocked
LEMMY_CONFIG_LOCATION=./docker/federation/lemmy_epsilon.hjson \
LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_epsilon" \
target/lemmy_server >/tmp/lemmy_epsilon.out 2>&1 &

echo "wait for all instances to start"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'lemmy-alpha:8541/api/v3/site')" != "200" ]]; do sleep 1; done
echo "alpha started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'lemmy-beta:8551/api/v3/site')" != "200" ]]; do sleep 1; done
echo "beta started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'lemmy-gamma:8561/api/v3/site')" != "200" ]]; do sleep 1; done
echo "gamma started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'lemmy-delta:8571/api/v3/site')" != "200" ]]; do sleep 1; done
echo "delta started"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'lemmy-epsilon:8581/api/v3/site')" != "200" ]]; do sleep 1; done
echo "epsilon started. All started"
