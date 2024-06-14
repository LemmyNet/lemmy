#!/usr/bin/env bash

pushd ../

# Check unused deps
cargo udeps --all-targets

# Update deps first
cargo update

# Upgrade deps
cargo upgrade

# Run clippy
cargo clippy

popd
