#!/usr/bin/env bash

set -e;

source "$HOME/.cargo/env"

case "$RUST_RELEASE_MODE" in
    "debug")
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > "crates/utils/src/version.rs"
        cargo build --features "${CARGO_BUILD_FEATURES}"
        cp "./target/$CARGO_BUILD_TARGET/$RUST_RELEASE_MODE/lemmy_server" /home/lemmy/lemmy_server
        ;;
    "release")
        # Pass a value to $USE_RELEASE_CACHE to avoid purging the cache for release builds
        [[ -z "$USE_RELEASE_CACHE" ]] || cargo clean --release
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > "crates/utils/src/version.rs"
        cargo build --features "${CARGO_BUILD_FEATURES}" --release
        cp "./target/$CARGO_BUILD_TARGET/$RUST_RELEASE_MODE/lemmy_server" /home/lemmy/lemmy_server
        ;;
esac
