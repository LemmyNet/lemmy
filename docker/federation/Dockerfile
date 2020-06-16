FROM ekidd/rust-musl-builder:1.42.0-openssl11

USER root
RUN mkdir /app/dist/documentation/ -p \
 && addgroup --gid 1001 lemmy \
 && adduser --disabled-password --shell /bin/sh -u 1001 --ingroup lemmy lemmy

# Copy resources
COPY server/config/defaults.hjson /app/config/defaults.hjson
COPY ui/dist /app/dist
COPY server/target/debug/lemmy_server /app/lemmy

RUN chown lemmy:lemmy /app/ -R
USER lemmy
EXPOSE 8536
WORKDIR /app
CMD ["/app/lemmy"]
