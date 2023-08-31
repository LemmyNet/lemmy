#!/usr/bin/env bash
set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd $CWD/../

cargo clippy --workspace --fix --allow-staged --allow-dirty --tests --all-targets --all-features -- \
  -D warnings -D deprecated -D clippy::perf -D clippy::complexity \
  -D clippy::style -D clippy::correctness -D clippy::suspicious \
  -D clippy::dbg_macro -D clippy::inefficient_to_string \
  -D clippy::items-after-statements -D clippy::implicit_clone \
  -D clippy::wildcard_imports -D clippy::cast_lossless \
  -D clippy::manual_string_new -D clippy::redundant_closure_for_method_calls \
  -D clippy::unused_self \
  -A clippy::uninlined_format_args \
  -D clippy::get_first \
  -D clippy::explicit_into_iter_loop \
  -D clippy::explicit_iter_loop \
  -D clippy::needless_collect \
  -D clippy::unwrap_used \
  -D clippy::indexing_slicing \
  -D clippy::needless_return

# Format rust files
cargo +nightly fmt

# Format toml files
taplo format

# Format sql files
find migrations -type f -name '*.sql' -exec pg_format -i {} +
