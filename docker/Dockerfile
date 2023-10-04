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

RUN rustup target add aarch64-unknown-linux-gnu

ARG CARGO_BUILD_FEATURES
ARG RUST_RELEASE_MODE

WORKDIR /lemmy

COPY . ./

# Debug build
RUN --mount=type=cache,target=/lemmy/target set -ex; \
    if [ "${RUST_RELEASE_MODE}" = "debug" ]; then \
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
        cargo build --target=aarch64-unknown-linux-gnu --features "${CARGO_BUILD_FEATURES}"; \
        mv target/aarch64-unknown-linux-gnu/debug/lemmy_server ./lemmy; \
    fi

# Release build
RUN set -ex; \
    if [ "${RUST_RELEASE_MODE}" = "release" ]; then \
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
        cargo build --target=aarch64-unknown-linux-gnu --features "${CARGO_BUILD_FEATURES}" --release; \
        mv target/aarch64-unknown-linux-gnu/release/lemmy_server ./lemmy; \
    fi

# Get target binary
FROM build-${TARGETARCH} AS build

## Final image
FROM ${RUNNER_IMAGE}

# Federation needs CA certificates
RUN apt update && apt install -y libssl-dev libpq-dev ca-certificates

# Debian / Ubuntu non-root user creds
ARG UNAME=lemmy
ARG UID=1000
ARG GID=1000
RUN groupadd -g $GID -o $UNAME
RUN useradd -m -u $UID -g $GID -o -s /bin/bash $UNAME
USER $UNAME

COPY --from=build /lemmy/lemmy ./
CMD ["./lemmy"]
EXPOSE 8536
STOPSIGNAL SIGTERM
