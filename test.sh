#!/bin/sh
export DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
diesel migration run
export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
RUST_TEST_THREADS=1 cargo test --workspace --no-fail-fast
