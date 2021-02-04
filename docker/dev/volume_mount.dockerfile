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

FROM ubuntu:20.10

# Install libpq for postgres and espeak
RUN apt-get update -y
RUN apt-get install -y libpq-dev espeak 

# Copy resources
COPY config/defaults.hjson /config/defaults.hjson
COPY --from=rust /app/lemmy_server /app/lemmy

EXPOSE 8536
CMD ["/app/lemmy"]
