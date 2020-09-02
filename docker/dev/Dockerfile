FROM node:10-jessie as node

WORKDIR /app/ui

# Cache deps
COPY ui/package.json ui/yarn.lock ./
RUN yarn install --pure-lockfile

# Build 
COPY ui /app/ui
RUN yarn build

FROM ekidd/rust-musl-builder:nightly-2020-05-07 as rust

# Cache deps
WORKDIR /app
RUN sudo chown -R rust:rust .
RUN USER=root cargo new server

# Install cargo-build-deps
RUN cargo install --git https://github.com/romac/cargo-build-deps.git
WORKDIR /app/server
RUN mkdir -p lemmy_db/src/ \
      lemmy_utils/src/ \
      lemmy_api_structs/src/ \
      lemmy_rate_limit/src/ \
      lemmy

# Copy the cargo tomls
COPY server/Cargo.toml server/Cargo.lock ./
COPY server/lemmy_db/Cargo.toml ./lemmy_db/
COPY server/lemmy_utils/Cargo.toml ./lemmy_utils/
COPY server/lemmy_api_structs/Cargo.toml ./lemmy_api_structs/
COPY server/lemmy_rate_limit/Cargo.toml ./lemmy_rate_limit/

# Cache the deps
RUN cargo build-deps

# Copy the src folders
COPY server/src ./src/
COPY server/lemmy_db/src ./lemmy_db/src/
COPY server/lemmy_utils/src/ ./lemmy_utils/src/
COPY server/lemmy_api_structs/src/ ./lemmy_api_structs/src/
COPY server/lemmy_rate_limit/src/ ./lemmy_rate_limit/src/
COPY server/migrations ./migrations/

# Build for debug
RUN cargo build

FROM ekidd/rust-musl-builder:nightly-2020-05-07 as docs
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
