# syntax=docker/dockerfile:experimental

# Warning: this will not pick up migrations unless there are code changes
FROM rust:1 as rust

ENV HOME=/home/root

WORKDIR /app

# Copy the source folders
COPY . ./
RUN echo "pub const VERSION: &str = \"$(git describe --tag)\";" > "crates/utils/src/version.rs"

# Build for debug
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build
RUN --mount=type=cache,target=/app/target \
    cp target/debug/lemmy_server lemmy_server

FROM ubuntu:20.04

# Install libpq for postgres
RUN apt-get update -y
RUN apt-get install -y libpq-dev ca-certificates

# Copy resources
COPY --from=rust /app/lemmy_server /app/lemmy

EXPOSE 8536
CMD ["/app/lemmy"]
