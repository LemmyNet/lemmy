#!/usr/bin/env bash
set -e

export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432
pushd ..
cargo build
rm target/lemmy_server || true
cp target/debug/lemmy_server target/lemmy_server
killall -s1 lemmy_server || true
./api_tests/prepare-drone-federation-test.sh
popd

pnpm i
pnpm api-test || true

killall -s1 lemmy_server || true
killall -s1 pict-rs || true
for INSTANCE in lemmy_alpha lemmy_beta lemmy_gamma lemmy_delta lemmy_epsilon; do
  psql "$LEMMY_DATABASE_URL" -c "DROP DATABASE $INSTANCE"
done
rm -r /tmp/pictrs
