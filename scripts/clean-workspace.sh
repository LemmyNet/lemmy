#!/bin/bash
set -e

# Run `cargo clean -p` for each workspace member. This allows to accurately measure the time for
# an incremental build.
clear && cargo metadata --no-deps | jq .packages.[].name | sed 's/.*/-p &/' | xargs cargo clean
