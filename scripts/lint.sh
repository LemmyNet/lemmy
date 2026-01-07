#!/usr/bin/env bash
set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd "$CWD/../"

# Format rust files
cargo +nightly fmt

# Format toml files
taplo format

# Format sql files
find migrations crates/diesel_utils/replaceable_schema -type f -name '*.sql' -print0 | xargs -0 -P 10 -L 10 pg_format -i

cargo clippy --workspace --fix --allow-staged --allow-dirty --tests --all-targets --all-features -- -D warnings
