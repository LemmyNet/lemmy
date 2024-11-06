#!/usr/bin/env bash
set -e

# This check is only used for CI.

CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cd "$CWD/../"

# Export the ts-rs bindings
cargo test --workspace export_bindings

# Make sure no rows are returned
! grep -nr --include=\*.ts ' | null' ./crates/
