# syntax=docker/dockerfile:experimental
FROM rust:1.47-buster as rust

ENV HOME=/home/root

WORKDIR /app

# Copy the source folders
COPY . ./

# Build for debug
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build
RUN --mount=type=cache,target=/app/target \
    cp target/debug/lemmy_server lemmy_server

FROM rust:1.47-buster as docs
WORKDIR /app
RUN cargo install mdbook --git https://github.com/Nutomic/mdBook.git \
        --branch localization --rev 0982a82 --force
COPY docs ./docs
RUN mdbook build docs/

FROM ubuntu:20.10

# Install libpq for postgres and espeak
RUN apt-get update -y
RUN apt-get install -y libpq-dev espeak 

# Copy resources
COPY config/defaults.hjson /config/defaults.hjson
COPY --from=rust /app/lemmy_server /app/lemmy
COPY --from=docs /app/docs/book/ /app/documentation/

EXPOSE 8536
CMD ["/app/lemmy"]
