# syntax=docker/dockerfile:1.6
ARG RUST_VERSION=1.72.1
ARG CARGO_BUILD_FEATURES=default
ARG RUST_RELEASE_MODE=debug

ARG AMD_BUILDER_IMAGE=rust:${RUST_VERSION}
ARG ARM_BUILDER_IMAGE="ghcr.io/raskyld/aarch64-lemmy-linux-gnu:v0.1.0"

ARG AMD_RUNNER_IMAGE=debian:bookworm-slim
ARG ARM_RUNNER_IMAGE=debian:bookwork-slim

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

FROM --platform=${BUILDPLATFORM} ${ARM_BUILDER_IMAGE} AS build-arm64

ARG RUST_RELEASE_MODE
ARG CARGO_BUILD_FEATURES

WORKDIR /home/lemmy/src
USER 10001:10001

COPY --chown=lemmy:lemmy . ./
COPY --chown=lemmy:lemmy --chmod=755 docker/docker-build.sh /home/lemmy/docker-build.sh

ENV RUST_RELEASE_MODE=${RUST_RELEASE_MODE} \
    CARGO_BUILD_FEATURES=${CARGO_BUILD_FEATURES}

RUN --mount=type=cache,target=./target,uid=10001,gid=10001 bash /home/lemmy/docker-build.sh

## Final image
FROM ${AMD_RUNNER_IMAGE}

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

# Arm Runner
FROM --platform=${BUILDPLATFORM} ${ARM_RUNNER_IMAGE}

ARG UNAME=lemmy
ARG UID=1000
ARG GID=1000

RUN apt update && apt install -y ca-certificates libssl-dev libpq-dev

COPY --from=build-arm64 --chmod=0755 /home/lemmy/lemmy_server /usr/local/bin

RUN addgroup -S -g ${GID} ${UNAME} && \
    adduser -S -H -D -G ${UNAME} -u ${UID} -g "" -s /sbin/nologin ${UNAME}
USER $UNAME

ENTRYPOINT ["lemmy_server"]
EXPOSE 8536
STOPSIGNAL SIGTERM
