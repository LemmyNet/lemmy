ARG RUST_BUILDER_IMAGE=ekidd/rust-musl-builder:1.47.0

# Cargo chef plan
FROM $RUST_BUILDER_IMAGE as planner
WORKDIR /app
RUN cargo install cargo-chef --version 0.1.6

# Copy dirs
COPY ./ ./

RUN sudo chown -R rust:rust .
RUN cargo chef prepare --recipe-path recipe.json

# Cargo chef cache dependencies
FROM $RUST_BUILDER_IMAGE as cacher
ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
WORKDIR /app
RUN cargo install cargo-chef --version 0.1.6
COPY --from=planner /app/recipe.json ./recipe.json
RUN sudo chown -R rust:rust .
RUN cargo chef cook --release --target ${CARGO_BUILD_TARGET} --recipe-path recipe.json

# Build the project
FROM $RUST_BUILDER_IMAGE as builder

ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
ARG RUSTRELEASEDIR="release"

WORKDIR /app

# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /home/rust/.cargo /home/rust/.cargo

# Copy the rest of the dirs
COPY ./ ./

RUN sudo chown -R rust:rust .
RUN cargo build --release

# reduce binary size
RUN strip ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server

RUN cp ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server /app/lemmy_server

# Build the docs
FROM $RUST_BUILDER_IMAGE as docs
WORKDIR /app
RUN cargo install mdbook --git https://github.com/Nutomic/mdBook.git \
        --branch localization --rev 0982a82 --force
COPY --chown=rust:rust docs ./docs
RUN mdbook build docs/

# The alpine runner
FROM alpine:3.12 as lemmy

# Install libpq for postgres
RUN apk add libpq

# Install Espeak for captchas
RUN apk add espeak

RUN addgroup -g 1000 lemmy
RUN adduser -D -s /bin/sh -u 1000 -G lemmy lemmy

# Copy resources
COPY --chown=lemmy:lemmy config/defaults.hjson /config/defaults.hjson
COPY --chown=lemmy:lemmy --from=builder /app/lemmy_server /app/lemmy
COPY --chown=lemmy:lemmy --from=docs /app/docs/book/ /app/documentation/

RUN chown lemmy:lemmy /app/lemmy
USER lemmy
EXPOSE 8536
CMD ["/app/lemmy"]
