
ARG RUST_BUILDER_IMAGE=shtripok/rust-musl-builder:arm

FROM $RUST_BUILDER_IMAGE as rust

#ARG RUSTRELEASEDIR="debug"
ARG RUSTRELEASEDIR="release"

# Cache deps
WORKDIR /app
RUN sudo chown -R rust:rust .
RUN USER=root cargo new server
WORKDIR /app/server
COPY --chown=rust:rust server/Cargo.toml server/Cargo.lock ./
#RUN sudo chown -R rust:rust .
RUN mkdir -p ./src/bin \
   && echo 'fn main() { println!("Dummy") }' > ./src/bin/main.rs
RUN cargo build --release
RUN rm -f ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/deps/lemmy_server*
COPY --chown=rust:rust server/src ./src/
COPY --chown=rust:rust server/migrations ./migrations/
#USER root
#RUN sudo chown -R rust:rust /app/server
#USER rust

# build for release
# workaround for https://github.com/rust-lang/rust/issues/62896
#RUN RUSTFLAGS='-Ccodegen-units=1' cargo build --release
RUN cargo build --frozen --release
#RUN cargo build --release

# Get diesel-cli on there just in case
# RUN cargo install diesel_cli --no-default-features --features postgres

RUN cp ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server /app/server/


FROM $RUST_BUILDER_IMAGE as docs
WORKDIR /app
COPY --chown=rust:rust docs ./docs
#RUN sudo chown -R rust:rust .
RUN mdbook build docs/


FROM node:12-buster as node

WORKDIR /app/ui

# Cache deps
COPY ui/package.json ui/yarn.lock ./
RUN yarn install --pure-lockfile --network-timeout 600000

# Build
COPY ui /app/ui
RUN yarn build


FROM alpine:3.10 as lemmy

# Install libpq for postgres
RUN apk add libpq
RUN addgroup -g 1000 lemmy
RUN adduser -D -s /bin/sh -u 1000 -G lemmy lemmy

# Copy resources
COPY --chown=lemmy:lemmy server/config/defaults.hjson /config/defaults.hjson
COPY --chown=lemmy:lemmy --from=rust /app/server/lemmy_server /app/lemmy
COPY --chown=lemmy:lemmy --from=docs /app/docs/book/ /app/dist/documentation/
COPY --chown=lemmy:lemmy --from=node /app/ui/dist /app/dist

RUN chown lemmy:lemmy /app/lemmy
USER lemmy
EXPOSE 8536
CMD ["/app/lemmy"]
