ARG RUST_BUILDER_IMAGE=clux/muslrust:1.59.0

FROM $RUST_BUILDER_IMAGE as chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

# Cargo chef plan
FROM chef as planner
ENV RUSTFLAGS="--cfg tokio_unstable"

# Copy dirs
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
ARG RUSTRELEASEDIR="debug"
ENV RUSTFLAGS="--cfg tokio_unstable"

COPY --from=planner /app/recipe.json ./recipe.json
RUN cargo chef cook --recipe-path recipe.json --target ${CARGO_BUILD_TARGET}

# Copy the rest of the dirs
COPY . .

# Build the project
RUN echo "pub const VERSION: &str = \"$(git describe --tag)\";" > "crates/utils/src/version.rs"
RUN cargo build --target ${CARGO_BUILD_TARGET}

# reduce binary size
RUN strip ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server

RUN cp ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server /app/lemmy_server

# The alpine runner
FROM alpine:3 as lemmy

# Install libpq for postgres
RUN apk add libpq

# Copy resources
COPY --from=builder /app/lemmy_server /app/lemmy

EXPOSE 8536
CMD ["/app/lemmy"]
