FROM node:12-buster as node

WORKDIR /app/ui

# Cache deps
COPY ui/package.json ui/yarn.lock ./
RUN yarn install --pure-lockfile --network-timeout 100000

# Build 
COPY ui /app/ui
RUN yarn build


FROM rust:1.37 as rust

# Cache deps
WORKDIR /app
RUN USER=root cargo new server
WORKDIR /app/server
COPY server/Cargo.toml server/Cargo.lock ./
RUN  mkdir -p ./src/bin \
  && echo 'fn main() { println!("Dummy") }' > ./src/bin/main.rs 

RUN cargo build
RUN rm -f ./target/debug/deps/lemmy_server*
COPY server/src ./src/
COPY server/migrations ./migrations/

# build for release
#RUN cargo build --frozen --release
RUN cargo build

# Get diesel-cli on there just in case
# RUN cargo install diesel_cli --no-default-features --features postgres

# make result place always the same for lemmy container
RUN cp /app/server/target/debug/lemmy_server /app/server/ready


#FROM alpine:3.10
# debian because build with dynamic linking with debian:buster
FROM debian:buster as lemmy

# Install libpq for postgres
#RUN apk add libpq
RUN apt-get update && apt-get install -y libpq5

# Copy resources
#COPY --from=rust /app/server/target/$TARGET/debug/lemmy_server /app/lemmy
COPY --from=rust /app/server/ready /app/lemmy
COPY --from=node /app/ui/dist /app/dist
RUN addgroup --gid 1000 lemmy
# for alpine
#RUN adduser -D -s /bin/sh -u 1000 -G lemmy lemmy
# for debian
RUN adduser --disabled-password --shell /bin/sh --uid 1000 --ingroup lemmy lemmy
RUN chown lemmy:lemmy /app/lemmy
USER lemmy
EXPOSE 8536
CMD ["/app/lemmy"]
