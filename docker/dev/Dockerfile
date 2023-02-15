FROM clux/muslrust:1.67.0 as builder
WORKDIR /app
ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
ARG RUSTRELEASEDIR="debug"

COPY . .

# Build the project
RUN echo "pub const VERSION: &str = \"$(git describe --tag)\";" > "crates/utils/src/version.rs"
RUN --mount=type=cache,target=/app/target \
    cargo build --target ${CARGO_BUILD_TARGET} \
     && cp ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server /app/lemmy_server

# The alpine runner
FROM alpine:3 as lemmy

# Install libpq for postgres
RUN apk add libpq

# Copy resources
COPY --from=builder /app/lemmy_server /app/lemmy

EXPOSE 8536
CMD ["/app/lemmy"]
