#!/bin/bash
set -e

export LEMMY_JWT_SECRET=changeme
export LEMMY_FEDERATION__ENABLED=true
export LEMMY_TLS_ENABLED=false
export LEMMY_SETUP__ADMIN_PASSWORD=lemmy
export LEMMY_RATE_LIMIT__POST=99999
export LEMMY_RATE_LIMIT__REGISTER=99999
export LEMMY_CAPTCHA__ENABLED=false
export LEMMY_TEST_SEND_SYNC=1
export RUST_BACKTRACE=1

for INSTANCE in lemmy_alpha lemmy_beta lemmy_gamma lemmy_delta lemmy_epsilon; do
  psql "${LEMMY_DATABASE_URL}/lemmy" -c "DROP DATABASE IF EXISTS $INSTANCE"
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

killall lemmy_server || true

echo "start alpha"
LEMMY_HOSTNAME=lemmy-alpha:8541 \
  LEMMY_PORT=8541 \
  LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_alpha" \
  LEMMY_FEDERATION__ALLOWED_INSTANCES=lemmy-beta,lemmy-gamma,lemmy-delta,lemmy-epsilon \
  LEMMY_SETUP__ADMIN_USERNAME=lemmy_alpha \
  LEMMY_SETUP__SITE_NAME=lemmy-alpha \
  target/lemmy_server >/dev/null 2>&1 &

echo "start beta"
LEMMY_HOSTNAME=lemmy-beta:8551 \
  LEMMY_PORT=8551 \
  LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_beta" \
  LEMMY_FEDERATION__ALLOWED_INSTANCES=lemmy-alpha,lemmy-gamma,lemmy-delta,lemmy-epsilon \
  LEMMY_SETUP__ADMIN_USERNAME=lemmy_beta \
  LEMMY_SETUP__SITE_NAME=lemmy-beta \
  target/lemmy_server >/dev/null 2>&1 &

echo "start gamma"
LEMMY_HOSTNAME=lemmy-gamma:8561 \
  LEMMY_PORT=8561 \
  LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_gamma" \
  LEMMY_FEDERATION__ALLOWED_INSTANCES=lemmy-alpha,lemmy-beta,lemmy-delta,lemmy-epsilon \
  LEMMY_SETUP__ADMIN_USERNAME=lemmy_gamma \
  LEMMY_SETUP__SITE_NAME=lemmy-gamma \
  target/lemmy_server >/dev/null 2>&1 &

echo "start delta"
# An instance with only an allowlist for beta
LEMMY_HOSTNAME=lemmy-delta:8571 \
  LEMMY_PORT=8571 \
  LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_delta" \
  LEMMY_FEDERATION__ALLOWED_INSTANCES=lemmy-beta \
  LEMMY_SETUP__ADMIN_USERNAME=lemmy_delta \
  LEMMY_SETUP__SITE_NAME=lemmy-delta \
  target/lemmy_server >/dev/null 2>&1 &

echo "start epsilon"
# An instance who has a blocklist, with lemmy-alpha blocked
LEMMY_HOSTNAME=lemmy-epsilon:8581 \
  LEMMY_PORT=8581 \
  LEMMY_DATABASE_URL="${LEMMY_DATABASE_URL}/lemmy_epsilon" \
  LEMMY_FEDERATION__BLOCKED_INSTANCES=lemmy-alpha \
  LEMMY_SETUP__ADMIN_USERNAME=lemmy_epsilon \
  LEMMY_SETUP__SITE_NAME=lemmy-epsilon \
  target/lemmy_server >/dev/null 2>&1 &

echo "wait for all instances to start"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8541/api/v2/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8551/api/v2/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8561/api/v2/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8571/api/v2/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8581/api/v2/site')" != "200" ]]; do sleep 1; done
