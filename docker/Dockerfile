# syntax=docker/dockerfile:1.6
ARG RUST_VERSION=1.72.1
ARG CARGO_BUILD_FEATURES=default
ARG RUST_RELEASE_MODE=debug
ARG LEMMY_VERSION="dev"

ARG AMD_BUILDER_IMAGE=rust:${RUST_VERSION}
ARG ARM_BUILDER_IMAGE="dessalines/lemmy-builder-arm64:${LEMMY_VERSION}"

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
# NB(raskyld): this is a hack to be able to COPY --from= this image, because the variable doesn't
# seem to be expended in --form arg of COPY :(
FROM ${ARM_RUNNER_IMAGE} AS build-arm64

# amd64 base runner
FROM ${AMD_RUNNER_IMAGE} AS runner-linux-amd64

# Federation needs CA certificates
RUN apt update && apt install -y libssl-dev libpq-dev ca-certificates

COPY --from=build-amd64 --chmod=0755 /lemmy/lemmy /usr/local/bin

# arm base runner
FROM ${ARM_RUNNER_IMAGE} AS runner-linux-arm

ARG ARM_BUILDER_IMAGE

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

RUN addgroup -S -g ${GID} ${UNAME} && \
    adduser -S -H -D -G ${UNAME} -u ${UID} -g "" -s /sbin/nologin ${UNAME}
USER $UNAME

ENTRYPOINT ["lemmy_server"]
EXPOSE 8536
STOPSIGNAL SIGTERM
