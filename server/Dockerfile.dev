# Setup env
FROM rust:1.33 AS build
RUN USER=root cargo new --bin /opt/lemmy/server--dev
WORKDIR /opt/lemmy/server--dev
# Enable deps caching
RUN mkdir -p src/bin
RUN echo 'fn main() { println!("Dummy") }' >src/bin/main.rs
# Install deps
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo build --release
RUN rm src/bin/main.rs
# Add app
COPY src/ src/
COPY migrations/ migrations/
RUN rm target/release/deps/lemmy*
RUN cargo build --release

# Setup env (no Alpine because Rust requires glibc)
FROM ubuntu:18.04
RUN apt update
RUN apt install postgresql-client -y
# Create empty directory where the frontend would normally be
RUN mkdir -p /opt/lemmy/ui--dev/dist
# Add app
COPY --from=build /opt/lemmy/server--dev/target/release/lemmy .
# Run app
CMD ["./lemmy"]
