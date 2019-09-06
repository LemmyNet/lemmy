FROM node:10-jessie as node
WORKDIR /app/ui

# Cache deps
COPY ui/package.json ui/yarn.lock ./
RUN yarn install --pure-lockfile

# Build 
COPY ui /app/ui
RUN yarn build

FROM rust:latest as rust

# Install musl
RUN apt-get update
RUN apt-get install musl-tools -y
RUN rustup target add x86_64-unknown-linux-musl

# Cache deps
WORKDIR /app
RUN USER=root cargo new server
WORKDIR /app/server
COPY server/Cargo.toml server/Cargo.lock ./
RUN  mkdir -p ./src/bin \
  && echo 'fn main() { println!("Dummy") }' > ./src/bin/main.rs 
RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl
RUN rm -f ./target/x86_64-unknown-linux-musl/release/deps/lemmy_server*
COPY server/src ./src/
COPY server/migrations ./migrations/

# build for release
RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --frozen --release --target=x86_64-unknown-linux-musl

# Get diesel-cli on there just in case
# RUN cargo install diesel_cli --no-default-features --features postgres

FROM alpine:latest

# Install libpq for postgres
RUN apk add libpq

# Copy resources
COPY --from=rust /app/server/target/x86_64-unknown-linux-musl/release/lemmy_server /app/lemmy
COPY --from=node /app/ui/dist /app/dist
RUN addgroup -g 1000 lemmy
RUN adduser -D -s /bin/sh -u 1000 -G lemmy lemmy
RUN chown lemmy:lemmy /app/lemmy
USER lemmy
EXPOSE 8536
CMD ["/app/lemmy"]
