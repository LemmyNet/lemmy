#!/usr/bin/env bash
set -e

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd $CWD/../

cargo clippy --workspace --fix --allow-staged --allow-dirty --tests --all-targets --all-features

# Format rust files
cargo +nightly fmt

# Format toml files
taplo format

# Format sql files
find migrations -type f -name '*.sql' -exec pg_format -i {} +
