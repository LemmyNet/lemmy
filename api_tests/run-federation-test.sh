#!/usr/bin/env bash
set -e

export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432

pushd ..
cargo build
rm target/lemmy_server || true
cp target/debug/lemmy_server target/lemmy_server
./api_tests/prepare-drone-federation-test.sh
popd

yarn
yarn api-test || true

killall lemmy_server

for INSTANCE in lemmy_alpha lemmy_beta lemmy_gamma lemmy_delta lemmy_epsilon; do
  psql "$LEMMY_DATABASE_URL" -c "DROP DATABASE $INSTANCE"
done
