ARG ARM_CROSS_TOOLCHAIN="ghcr.io/raskyld/aarch64-lemmy-linux-gnu:v0.1.0"

FROM ${ARM_CROSS_TOOLCHAIN}

# NB(raskyld): Please, do not hesitate to contact me through @raskyld@social.vivaldi.net (mastodon)
# If you have any question about the cross-toolchain
LABEL org.opencontainers.image.authors="Enzo Nocera <enzo@nocera.eu>"
LABEL org.opencontainers.image.source="https://github.com/LemmyNet/lemmy"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"
LABEL org.opencontainers.image.description="A prebuilt lemmy server using a cross toolchain from amd64 to aarch64/arm64"

ARG CARGO_BUILD_FEATURES=default
ARG RUST_RELEASE_MODE=debug

WORKDIR /home/lemmy/src
USER 10001:10001

COPY --chown=lemmy:lemmy . ./

ENV RUST_RELEASE_MODE=${RUST_RELEASE_MODE} \
    CARGO_BUILD_FEATURES=${CARGO_BUILD_FEATURES}

RUN --mount=type=cache,target=./target,uid=10001,gid=10001 bash ./docker/builders/lemmy-builder-arm64/docker-build.sh
