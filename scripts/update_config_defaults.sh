#!/usr/bin/env bash
set -e

dest=${1-config/defaults.hjson}

cargo run --manifest-path crates/utils/Cargo.toml --features full > "$dest"
