#!/usr/bin/env bash

pushd ../

# Check unused deps
cargo udeps --all-targets

# Upgrade deps
cargo upgrade

# Run clippy
cargo clippy

popd
