# Build the project
FROM clux/muslrust:1.59.0 as builder

ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
ARG RUSTRELEASEDIR="release"

WORKDIR /app

COPY ./ ./

RUN echo "pub const VERSION: &str = \"$(git describe --tag)\";" > "crates/utils/src/version.rs"
RUN cargo build --release

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
