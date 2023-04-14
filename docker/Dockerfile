FROM rust:1.67.0-alpine as builder

# Install build dependencies
RUN apk add --no-cache git openssl-dev libpq-dev musl-dev

# Set the working directory to /app and copy the source code
WORKDIR /app
COPY . .

# Set cargo build target (can be changed using --build-arg)
ARG CARGO_BUILD_TARGET="x86_64-unknown-linux-musl"
# Set the release mode (can be changed using --build-arg)
ARG RUST_RELEASE_MODE="debug"

# Debug mode build
RUN --mount=type=cache,target=/app/target \
    if [ "$RUST_RELEASE_MODE" = "debug" ]; then \
      echo "pub const VERSION: &str = \"$(git describe --tag)\";" > "crates/utils/src/version.rs" \
      && rustup target add ${CARGO_BUILD_TARGET} \
      && cargo build --target ${CARGO_BUILD_TARGET} \
      && cp ./target/${CARGO_BUILD_TARGET}/${RUST_RELEASE_MODE}/lemmy_server /app/lemmy_server; \
    fi

# Release mode build
RUN \
    if [ "$RUST_RELEASE_MODE" = "release" ]; then \
      rustup target add ${CARGO_BUILD_TARGET} \
      && cargo build --target ${CARGO_BUILD_TARGET} --release \
      && cp ./target/${CARGO_BUILD_TARGET}/${RUST_RELEASE_MODE}/lemmy_server /app/lemmy_server; \
    fi

# The Alpine runner
FROM alpine:3 as lemmy

# Install libpq for Postgres
RUN apk add --no-cache ca-certificates libpq

# Copy resources
COPY --from=builder /app/lemmy_server /app/lemmy

EXPOSE 8536
CMD ["/app/lemmy"]
