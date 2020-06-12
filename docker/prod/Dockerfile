FROM node:10-jessie as node

WORKDIR /app/ui

# Cache deps
COPY ui/package.json ui/yarn.lock ./
RUN yarn install --pure-lockfile

# Build 
COPY ui /app/ui
RUN yarn build

FROM ekidd/rust-musl-builder:1.42.0-openssl11 as rust

# Install lld
RUN sudo apt-get update -y 
RUN sudo apt-get install -y lld 

# Cache deps
WORKDIR /app
RUN sudo chown -R rust:rust .
RUN USER=root cargo new server
WORKDIR /app/server
COPY server/Cargo.toml server/Cargo.lock ./
RUN sudo chown -R rust:rust .
RUN mkdir -p ./src/bin \
  && echo 'fn main() { println!("Dummy") }' > ./src/bin/main.rs 
RUN RUSTFLAGS="-Clink-arg=-fuse-ld=lld" cargo build --release
RUN rm -f ./target/x86_64-unknown-linux-musl/release/deps/lemmy_server*
COPY server/src ./src/
COPY server/migrations ./migrations/

# Build for release
RUN RUSTFLAGS="-Clink-arg=-fuse-ld=lld" cargo build --frozen --release

# reduce binary size
RUN strip /app/server/target/x86_64-unknown-linux-musl/release/lemmy_server

FROM ekidd/rust-musl-builder:1.42.0-openssl11 as docs
WORKDIR /app
COPY docs ./docs
RUN sudo chown -R rust:rust .
RUN mdbook build docs/


FROM alpine:3.10

# Install libpq for postgres
RUN apk add libpq

# Copy resources
COPY server/config/defaults.hjson /config/defaults.hjson
COPY --from=rust /app/server/target/x86_64-unknown-linux-musl/release/lemmy_server /app/lemmy
COPY --from=docs /app/docs/book/ /app/dist/documentation/
COPY --from=node /app/ui/dist /app/dist

RUN addgroup -g 1000 lemmy
RUN adduser -D -s /bin/sh -u 1000 -G lemmy lemmy
RUN chown lemmy:lemmy /app/lemmy
USER lemmy
EXPOSE 8536
CMD ["/app/lemmy"]
