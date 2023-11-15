#!/usr/bin/env bash
set -e

# pictrs setup
if ! [ -f "pict-rs" ]; then
  curl "https://git.asonix.dog/asonix/pict-rs/releases/download/v0.5.0-beta.2/pict-rs-linux-amd64" -o pict-rs
  chmod +x pict-rs
fi
./pict-rs \
  run -a 0.0.0.0:8080 \
  --danger-dummy-mode \
  filesystem -p /tmp/pictrs/files \
  sled -p /tmp/pictrs/sled-repo 2>&1 &

export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432
pushd ..
cargo build
rm target/lemmy_server || true
cp target/debug/lemmy_server target/lemmy_server
killall -s1 lemmy_server || true
./api_tests/prepare-drone-federation-test.sh
popd

yarn
yarn api-test || true

killall -s1 lemmy_server || true
killall -s1 pict-rs || true
for INSTANCE in lemmy_alpha lemmy_beta lemmy_gamma lemmy_delta lemmy_epsilon; do
  psql "$LEMMY_DATABASE_URL" -c "DROP DATABASE $INSTANCE"
done
rm -r /tmp/pictrs
