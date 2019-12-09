#!/bin/sh
set -e

export DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
export JWT_SECRET=changeme
export HOSTNAME=rrr

cd ui
yarn
yarn build
cd ../server
cargo run

# For live coding, where both the front and back end, automagically reload on any save, do:
# cd ui && yarn start
# cd server && cargo watch -x run
