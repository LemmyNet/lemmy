ARG RUST_VERSION=1.72.1
ARG CARGO_BUILD_FEATURES=default
ARG RUST_RELEASE_MODE=debug
ARG BUILDER_IMAGE=rust:${RUST_VERSION}
ARG RUNNER_IMAGE=debian:bookworm-slim

# AMD64 builder
FROM --platform=${BUILDPLATFORM} ${BUILDER_IMAGE} AS build-amd64

ARG CARGO_BUILD_FEATURES
ARG RUST_RELEASE_MODE

WORKDIR /lemmy

COPY . ./

# Debug build
RUN --mount=type=cache,target=/lemmy/target set -ex; \
    if [ "${RUST_RELEASE_MODE}" = "debug" ]; then \
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
        cargo build --features "${CARGO_BUILD_FEATURES}"; \
        mv target/debug/lemmy_server ./lemmy; \
    fi

# Release build
RUN set -ex; \
    if [ "${RUST_RELEASE_MODE}" = "release" ]; then \
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
        cargo build --features "${CARGO_BUILD_FEATURES}" --release; \
        mv target/release/lemmy_server ./lemmy; \
    fi

# ARM64 builder
FROM --platform=${BUILDPLATFORM} ${BUILDER_IMAGE} AS build-arm64

ARG CARGO_BUILD_FEATURES
ARG RUST_RELEASE_MODE

WORKDIR /lemmy

COPY . ./

# Debug build
RUN --mount=type=cache,target=/lemmy/target set -ex; \
    if [ "${RUST_RELEASE_MODE}" = "debug" ]; then \
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
        cargo build --target=aarch64-unknown-linux-musl --features "${CARGO_BUILD_FEATURES}"; \
        mv target/aarch64-unknown-linux-musl/debug/lemmy_server ./lemmy; \
    fi

# Release build
RUN set -ex; \
    if [ "${RUST_RELEASE_MODE}" = "release" ]; then \
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
        cargo build --target=aarch64-unknown-linux-musl --features "${CARGO_BUILD_FEATURES}" --release; \
        mv target/aarch64-unknown-linux-musl/release/lemmy_server ./lemmy; \
    fi

# Get target binary
FROM build-${TARGETARCH} AS build

## Final image
FROM ${RUNNER_IMAGE}

RUN apt update && apt install -y libssl-dev libpq-dev

COPY --from=build /lemmy/lemmy /app/
CMD ["/app/lemmy"]
EXPOSE 8536
STOPSIGNAL SIGTERM
