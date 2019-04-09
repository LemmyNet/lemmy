FROM node:10-jessie as node
#If encounter Invalid cross-device error -run on host 'echo N | sudo tee /sys/module/overlay/parameters/metacopy'
COPY ui /app/ui
RUN cd /app/ui && yarn && yarn build

FROM rust:1.33 as rust
COPY server /app/server
WORKDIR /app/server
COPY --from=node /app/ui/dist /app/dist
RUN cargo build --release
RUN mv /app/server/target/release/lemmy /app/
EXPOSE 8536
