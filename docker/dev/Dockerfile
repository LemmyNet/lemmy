ARG RUST_BUILDER_IMAGE=ekidd/rust-musl-builder:1.51.0

# Cargo chef plan
FROM $RUST_BUILDER_IMAGE as planner
WORKDIR /app
RUN cargo install cargo-chef

# Copy dirs
COPY ./ ./

RUN sudo chown -R rust:rust .
RUN cargo chef prepare --recipe-path recipe.json

# Cargo chef cache dependencies
FROM $RUST_BUILDER_IMAGE as cacher
ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json ./recipe.json
RUN sudo chown -R rust:rust .
RUN cargo chef cook --target ${CARGO_BUILD_TARGET} --recipe-path recipe.json

# Build the project
FROM $RUST_BUILDER_IMAGE as builder

ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
ARG RUSTRELEASEDIR="debug"

WORKDIR /app

# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /home/rust/.cargo /home/rust/.cargo

# Copy the rest of the dirs
COPY ./ ./

RUN sudo chown -R rust:rust .
RUN echo "pub const VERSION: &str = \"$(git describe --tag)\";" > "crates/utils/src/version.rs"
RUN cargo build

# reduce binary size
RUN strip ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server

RUN cp ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server /app/lemmy_server

# The alpine runner
FROM alpine:3 as lemmy

# Install libpq for postgres
RUN apk add libpq

RUN addgroup -g 1000 lemmy
RUN adduser -D -s /bin/sh -u 1000 -G lemmy lemmy

# Copy resources
COPY --chown=lemmy:lemmy --from=builder /app/lemmy_server /app/lemmy

RUN chown lemmy:lemmy /app/lemmy
USER lemmy
EXPOSE 8536
CMD ["/app/lemmy"]
