#!/usr/bin/env bash
set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd $CWD/../

# Format rust files
cargo +nightly fmt

# Format toml files
taplo format

# Format sql files
find migrations crates/db_schema/replaceable_schema -type f -name '*.sql' -exec pg_format -i {} +

cargo clippy --workspace --fix --allow-staged --allow-dirty --tests --all-targets --all-features -- -D warnings
