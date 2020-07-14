FROM node:10-jessie as node

WORKDIR /app/ui

# Cache deps
COPY ui/package.json ui/yarn.lock ./
RUN yarn install --pure-lockfile

# Build 
COPY ui /app/ui
RUN yarn build

FROM ekidd/rust-musl-builder:1.42.0-openssl11 as rust

# Cache deps
WORKDIR /app
RUN sudo chown -R rust:rust .
RUN USER=root cargo new server
WORKDIR /app/server
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/lemmy_db ./lemmy_db
COPY server/lemmy_utils ./lemmy_utils
RUN mkdir -p ./src/bin \
   && echo 'fn main() { println!("Dummy") }' > ./src/bin/main.rs
RUN cargo build
RUN find target/debug -type f -name "$(echo "lemmy_server" | tr '-' '_')*" -exec touch -t 200001010000 {} +
COPY server/src ./src/
COPY server/migrations ./migrations/

# Build for debug
RUN cargo build

FROM ekidd/rust-musl-builder:1.42.0-openssl11 as docs
WORKDIR /app
COPY docs ./docs
RUN sudo chown -R rust:rust .
RUN mdbook build docs/

FROM alpine:3.12

# Install libpq for postgres
RUN apk add libpq

# Copy resources
COPY server/config/defaults.hjson /config/defaults.hjson
COPY --from=rust /app/server/target/x86_64-unknown-linux-musl/debug/lemmy_server /app/lemmy
COPY --from=docs /app/docs/book/ /app/dist/documentation/
COPY --from=node /app/ui/dist /app/dist

RUN addgroup -g 1000 lemmy
RUN adduser -D -s /bin/sh -u 1000 -G lemmy lemmy
RUN chown lemmy:lemmy /app/lemmy
USER lemmy
EXPOSE 8536
CMD ["/app/lemmy"]
