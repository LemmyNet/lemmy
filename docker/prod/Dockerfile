ARG RUST_BUILDER_IMAGE=ekidd/rust-musl-builder:stable

FROM $RUST_BUILDER_IMAGE as rust

ARG CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
ARG RUSTRELEASEDIR="release"

WORKDIR /app/server
RUN sudo chown -R rust:rust .
COPY . ./
RUN cargo build --release

# reduce binary size
RUN strip ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server

RUN cp ./target/$CARGO_BUILD_TARGET/$RUSTRELEASEDIR/lemmy_server /app/server/

FROM $RUST_BUILDER_IMAGE as docs
WORKDIR /app
COPY --chown=rust:rust docs ./docs
RUN mdbook build docs/

FROM alpine:3.12 as lemmy

# Install libpq for postgres
RUN apk add libpq

# Install Espeak for captchas
RUN apk add espeak

RUN addgroup -g 1000 lemmy
RUN adduser -D -s /bin/sh -u 1000 -G lemmy lemmy

# Copy resources
COPY --chown=lemmy:lemmy config/defaults.hjson /config/defaults.hjson
COPY --chown=lemmy:lemmy --from=rust /app/server/lemmy_server /app/lemmy
COPY --chown=lemmy:lemmy --from=docs /app/docs/book/ /app/documentation/

RUN chown lemmy:lemmy /app/lemmy
USER lemmy
EXPOSE 8536
CMD ["/app/lemmy"]
