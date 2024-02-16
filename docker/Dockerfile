# syntax=docker/dockerfile:1.6
ARG RUST_VERSION=1.76
ARG CARGO_BUILD_FEATURES=default
ARG RUST_RELEASE_MODE=debug

ARG AMD_BUILDER_IMAGE=rust:${RUST_VERSION}
# Repo: https://github.com/raskyld/lemmy-cross-toolchains
ARG ARM_BUILDER_IMAGE="ghcr.io/raskyld/aarch64-lemmy-linux-gnu:v0.2.0"

ARG AMD_RUNNER_IMAGE=debian:bookworm-slim
ARG ARM_RUNNER_IMAGE=debian:bookworm-slim

ARG UNAME=lemmy
ARG UID=1000
ARG GID=1000

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
        mv target/"${RUST_RELEASE_MODE}"/lemmy_server ./lemmy_server; \
    fi

# Release build
RUN --mount=type=cache,target=/lemmy/target set -ex; \
    if [ "${RUST_RELEASE_MODE}" = "release" ]; then \
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
        [ -z "$USE_RELEASE_CACHE" ] && cargo clean --release; \
        cargo build --features "${CARGO_BUILD_FEATURES}" --release; \
        mv target/"${RUST_RELEASE_MODE}"/lemmy_server ./lemmy_server; \
    fi

# ARM64 builder
# NB(raskyld): this is a hack to be able to COPY --from= this image, because the variable doesn't
# seem to be expended in --form arg of COPY :(
FROM --platform=linux/amd64 ${ARM_BUILDER_IMAGE} AS build-arm64

ARG RUST_RELEASE_MODE
ARG CARGO_BUILD_FEATURES

WORKDIR /home/lemmy/src
USER 10001:10001

COPY --chown=lemmy:lemmy . ./

ENV PATH="/home/lemmy/.cargo/bin:${PATH}"
ENV RUST_RELEASE_MODE=${RUST_RELEASE_MODE} \
    CARGO_BUILD_FEATURES=${CARGO_BUILD_FEATURES}

# Debug build
RUN --mount=type=cache,target=./target,uid=10001,gid=10001 set -ex; \
    if [ "${RUST_RELEASE_MODE}" = "debug" ]; then \
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
        cargo build --features "${CARGO_BUILD_FEATURES}"; \
        mv "./target/$CARGO_BUILD_TARGET/$RUST_RELEASE_MODE/lemmy_server" /home/lemmy/lemmy_server; \
    fi

# Release build
RUN --mount=type=cache,target=./target,uid=10001,gid=10001 set -ex; \
    if [ "${RUST_RELEASE_MODE}" = "release" ]; then \
        echo "pub const VERSION: &str = \"$(git describe --tag)\";" > crates/utils/src/version.rs; \
        [ -z "$USE_RELEASE_CACHE" ] && cargo clean --release; \
        cargo build --features "${CARGO_BUILD_FEATURES}" --release; \
        mv "./target/$CARGO_BUILD_TARGET/$RUST_RELEASE_MODE/lemmy_server" /home/lemmy/lemmy_server; \
    fi

# amd64 base runner
FROM ${AMD_RUNNER_IMAGE} AS runner-linux-amd64

# Federation needs CA certificates
RUN apt update && apt install -y libssl-dev libpq-dev ca-certificates

COPY --from=build-amd64 --chmod=0755 /lemmy/lemmy_server /usr/local/bin

# arm base runner
FROM ${ARM_RUNNER_IMAGE} AS runner-linux-arm64

RUN apt update && apt install -y ca-certificates libssl-dev libpq-dev

COPY --from=build-arm64 --chmod=0755 /home/lemmy/lemmy_server /usr/local/bin

# Final image that use a base runner based on the target OS and ARCH
FROM runner-${TARGETOS}-${TARGETARCH}

LABEL org.opencontainers.image.authors="The Lemmy Authors"
LABEL org.opencontainers.image.source="https://github.com/LemmyNet/lemmy"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
LABEL org.opencontainers.image.description="A link aggregator and forum for the fediverse"

ARG UNAME
ARG GID
ARG UID

RUN groupadd -g ${GID} -o ${UNAME} && \
    useradd -m -u ${UID} -g ${GID} -o -s /bin/bash ${UNAME}
USER $UNAME

ENTRYPOINT ["lemmy_server"]
EXPOSE 8536
STOPSIGNAL SIGTERM
