#!/usr/bin/env bash

pushd ../

# Check unused deps
cargo udeps --all-targets

# Upgrade deps
cargo upgrade --workspace

# Run check
cargo check

popd
