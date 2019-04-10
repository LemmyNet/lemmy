FROM node:10-jessie as node
#If encounter Invalid cross-device error -run on host 'echo N | sudo tee /sys/module/overlay/parameters/metacopy'
COPY ui /app/ui
WORKDIR /app/ui
RUN yarn
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
RUN cargo build --release --bin lemmy
RUN ls ./target/release/.fingerprint/
RUN rm -r ./target/release/.fingerprint/server-*

# copy your source tree
# RUN rm -rf ./src/
COPY server/src ./src/
COPY server/migrations ./migrations/

# build for release
RUN cargo build --frozen --release --bin lemmy
RUN mv /app/server/target/release/lemmy /app/lemmy

# The output image
# FROM debian:stable-slim
# RUN apt-get -y update && apt-get install -y postgresql-client
# COPY --from=rust /app/server/target/release/lemmy /app/lemmy
COPY --from=node /app/ui/dist /app/dist
EXPOSE 8536
