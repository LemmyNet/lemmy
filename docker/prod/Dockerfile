# Build the project
FROM ekidd/rust-musl-builder:1.51.0 as builder

ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
ARG RUSTRELEASEDIR="release"

WORKDIR /app

COPY ./ ./

RUN sudo chown -R rust:rust .
RUN echo "pub const VERSION: &str = \"$(git describe --tag)\";" > "crates/utils/src/version.rs"
RUN cargo build --release

# reduce binary size
RUN strip ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server

RUN cp ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server /app/lemmy_server

# The alpine runner
FROM alpine:3.14 as lemmy

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
