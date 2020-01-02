FROM ekidd/rust-musl-builder:1.38.0-openssl11

# Copy resources
COPY server/config/defaults.hjson /app/config/defaults.hjson
COPY server/target/debug/lemmy_server /app/lemmy
COPY ui/dist /app/dist

USER root
RUN mkdir /app/dist/documentation/
RUN addgroup --gid 1001 lemmy
RUN adduser --disabled-password --shell /bin/sh -u 1001 --ingroup lemmy lemmy
RUN chown lemmy:lemmy /app/ -R
USER lemmy
EXPOSE 8536
WORKDIR /app
CMD ["/app/lemmy"]
