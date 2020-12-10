#!/bin/bash
set -e

# change folder so the config can be read from the default location
cd ..

export LEMMY_JWT_SECRET=changeme
export LEMMY_FEDERATION__ENABLED=true
export LEMMY_TLS_ENABLED=false
export LEMMY_SETUP__ADMIN_PASSWORD=lemmy
export LEMMY_RATE_LIMIT__POST=99999
export LEMMY_RATE_LIMIT__REGISTER=99999
export LEMMY_CAPTCHA__ENABLED=false
export RUST_BACKTRACE=1
export RUST_LOG=debug

echo "start alpha"
LEMMY_HOSTNAME=lemmy-alpha:8541 \
  LEMMY_PORT=8541 \
  LEMMY_DATABASE_URL=postgres://lemmy:password@postgres_alpha:5432/lemmy \
  LEMMY_FEDERATION__ALLOWED_INSTANCES=lemmy-beta,lemmy-gamma,lemmy-delta,lemmy-epsilon \
  LEMMY_SETUP__ADMIN_USERNAME=lemmy_alpha \
  LEMMY_SETUP__SITE_NAME=lemmy-alpha \
  target/debug/lemmy_server &

echo "start beta"
LEMMY_HOSTNAME=lemmy-beta:8551 \
  LEMMY_PORT=8551 \
  LEMMY_DATABASE_URL=postgres://lemmy:password@postgres_beta:5432/lemmy \
  LEMMY_FEDERATION__ALLOWED_INSTANCES=lemmy-alpha,lemmy-gamma,lemmy-delta,lemmy-epsilon \
  LEMMY_SETUP__ADMIN_USERNAME=lemmy_beta \
  LEMMY_SETUP__SITE_NAME=lemmy-beta \
  target/debug/lemmy_server &

echo "start gamma"
LEMMY_HOSTNAME=lemmy-gamma:8561 \
  LEMMY_PORT=8561 \
  LEMMY_DATABASE_URL=postgres://lemmy:password@postgres_gamma:5432/lemmy \
  LEMMY_FEDERATION__ALLOWED_INSTANCES=lemmy-alpha,lemmy-beta,lemmy-delta,lemmy-epsilon \
  LEMMY_SETUP__ADMIN_USERNAME=lemmy_gamma \
  LEMMY_SETUP__SITE_NAME=lemmy-gamma \
  target/debug/lemmy_server &

echo "start delta"
# An instance with only an allowlist for beta
LEMMY_HOSTNAME=lemmy-delta:8571 \
  LEMMY_PORT=8571 \
  LEMMY_DATABASE_URL=postgres://lemmy:password@postgres_delta:5432/lemmy \
  LEMMY_FEDERATION__ALLOWED_INSTANCES=lemmy-beta \
  LEMMY_SETUP__ADMIN_USERNAME=lemmy_delta \
  LEMMY_SETUP__SITE_NAME=lemmy-delta \
  target/debug/lemmy_server &

echo "start epsilon"
# An instance who has a blocklist, with lemmy-alpha blocked
LEMMY_HOSTNAME=lemmy-epsilon:8581 \
  LEMMY_PORT=8581 \
  LEMMY_DATABASE_URL=postgres://lemmy:password@postgres_epsilon:5432/lemmy \
  LEMMY_FEDERATION__BLOCKED_INSTANCES=lemmy-alpha \
  LEMMY_SETUP__ADMIN_USERNAME=lemmy_epsilon \
  LEMMY_SETUP__SITE_NAME=lemmy-epsilon \
  target/debug/lemmy_server &

echo "wait for all instances to start"
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8541/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8551/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8561/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8571/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8581/api/v1/site')" != "200" ]]; do sleep 1; done
