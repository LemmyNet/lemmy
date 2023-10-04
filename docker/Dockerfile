ARG RUST_VERSION=1.72.1
ARG CARGO_BUILD_FEATURES=default
ARG RUST_RELEASE_MODE=debug
ARG AMD_BUILDER_IMAGE=rust:${RUST_VERSION}
ARG ARM_BUILDER_IMAGE=blackdex/rust-musl:aarch64-musl-stable-${RUST_VERSION}-openssl3
ARG AMD_RUNNER_IMAGE=debian:bookworm-slim
ARG ARM_RUNNER_IMAGE=alpine:3.18

# AMD64 builder
FROM --platform=${BUILDPLATFORM} ${AMD_BUILDER_IMAGE} AS build-amd64

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
# TODO currently broken
# FROM --platform=${BUILDPLATFORM} ${ARM_BUILDER_IMAGE} as build-arm64

# ENV DEBIAN_FRONTEND=noninteractive
# ENV CARGO_HOME=/root/.cargo
# ENV PQ_LIB_DIR=/usr/local/musl/pq15/lib

# RUN apt update && apt install -y \
#     --no-install-recommends \
#     git

# RUN mkdir -pv "${CARGO_HOME}" && \
#     rustup set profile minimal && \
#     rustup target add aarch64-unknown-linux-musl

# ARG CARGO_BUILD_FEATURES
# ARG RUST_RELEASE_MODE

# WORKDIR /lemmy

# COPY . ./

# # Debug build
# RUN --mount=type=cache,target=/lemmy/target set -ex; \
#     if [ "${RUST_RELEASE_MODE}" = "debug" ]; then \
#         echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
#         cargo build --target=aarch64-unknown-linux-musl --features "${CARGO_BUILD_FEATURES}"; \
#         mv target/aarch64-unknown-linux-musl/debug/lemmy_server ./lemmy; \
#     fi

# # Release build
# RUN set -ex; \
#     if [ "${RUST_RELEASE_MODE}" = "release" ]; then \
#         echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
#         cargo build --target=aarch64-unknown-linux-musl --features "${CARGO_BUILD_FEATURES}" --release; \
#         mv target/aarch64-unknown-linux-musl/release/lemmy_server ./lemmy; \
#     fi

## Final image
FROM --platform=${BUILDPLATFORM} ${AMD_RUNNER_IMAGE}

# Federation needs CA certificates
RUN apt update && apt install -y libssl-dev libpq-dev ca-certificates

# Debian / Ubuntu non-root user creds
ARG UNAME=lemmy
ARG UID=1000
ARG GID=1000
RUN groupadd -g $GID -o $UNAME
RUN useradd -m -u $UID -g $GID -o -s /bin/bash $UNAME
USER $UNAME

COPY --from=build-amd64 /lemmy/lemmy ./
CMD ["./lemmy"]
EXPOSE 8536
STOPSIGNAL SIGTERM

## Arm Runner
# FROM --platform=${BUILDPLATFORM} ${ARM_RUNNER_IMAGE}

# ARG UNAME=lemmy
# ARG UID=1000
# ARG GID=1000

# RUN apk add --no-cache ca-certificates

# COPY --from=build-arm64 --chmod=0755 /lemmy/lemmy /usr/local/bin

# RUN addgroup -S -g ${GID} ${UNAME} && \
#     adduser -S -H -D -G ${UNAME} -u ${UID} -g "" -s /sbin/nologin ${UNAME}
# USER $UNAME

# CMD ["lemmy"]
# EXPOSE 8536
# STOPSIGNAL SIGTERM

