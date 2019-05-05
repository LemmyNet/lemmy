FROM node:10-jessie as node
#If encounter Invalid cross-device error -run on host 'echo N | sudo tee /sys/module/overlay/parameters/metacopy'
WORKDIR /app/ui

COPY ui/package.json ui/yarn.lock ./
RUN yarn install --pure-lockfile # This caches your deps
COPY ui /app/ui
RUN yarn build

FROM rust:1.33 as rust

# create a new empty shell project
WORKDIR /app
RUN USER=root cargo new server
WORKDIR /app/server

# copy over your manifests
COPY server/Cargo.toml server/Cargo.lock ./

# this build step will cache your dependencies
RUN  mkdir -p ./src/bin \
  && echo 'fn main() { println!("Dummy") }' > ./src/bin/main.rs 
RUN cargo build --release
RUN rm -r ./target/release/.fingerprint/lemmy_server-*

# copy your source tree
# RUN rm -rf ./src/
COPY server/src ./src/
COPY server/migrations ./migrations/

# build for release
RUN cargo build --frozen --release
RUN mv /app/server/target/release/lemmy_server /app/lemmy

# Get diesel-cli on there just in case
# RUN cargo install diesel_cli --no-default-features --features postgres

# The output image
# FROM debian:stable-slim
# RUN apt-get -y update && apt-get install -y postgresql-client
# COPY --from=rust /app/server/target/release/lemmy /app/lemmy
COPY --from=node /app/ui/dist /app/dist
EXPOSE 8536
