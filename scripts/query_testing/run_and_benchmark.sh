#!/usr/bin/env bash
set -e

LEMMY_BENCHMARK=1 cargo build --release

RUST_LOG=error target/release/lemmy_server  &

# Wait for port to be opened by server
sleep 3

scripts/query_testing/api_benchmark.sh

kill $!
