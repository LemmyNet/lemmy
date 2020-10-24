# syntax=docker/dockerfile:experimental
FROM rust:1.47-buster as rust

ENV HOME=/home/root

WORKDIR /app

# Copy the source folders
COPY . .

# Build for debug
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build
RUN --mount=type=cache,target=/app/target \
    cp target/debug/lemmy_server lemmy_server

FROM ekidd/rust-musl-builder:1.47.0 as docs
WORKDIR /app
COPY docs ./docs
RUN sudo chown -R rust:rust .
RUN mdbook build docs/

FROM alpine:3.12

# Install libpq for postgres
RUN apk add libpq

# Install Espeak for captchas
RUN apk add espeak

# Copy resources
COPY config/defaults.hjson /config/defaults.hjson
COPY --from=rust /app/lemmy_server /app/lemmy
COPY --from=docs /app/docs/book/ /app/documentation/

RUN addgroup -g 1000 lemmy
RUN adduser -D -s /bin/sh -u 1000 -G lemmy lemmy
RUN chown lemmy:lemmy /app/lemmy
USER lemmy
EXPOSE 8536
CMD ["/app/lemmy"]
