#!/usr/bin/env bash
set -e

dest=${1-config/defaults.hjson}

cargo run -- --print-config-docs > "$dest"
