#!/bin/sh
set -e

if [ "$1" = '/app/lemmy' ]; then
    # Default database settings
    : "${DB_USER:=lemmy}"
    : "${DB_PASSWORD:=password}"
    : "${DB_HOST:=localhost}"
    : "${DB_PORT:=5432}"
    : "${DB_DATABASE:=lemmy}"
    : "${DB_POOL_SIZE:=5}"

    # Default app settings
    : "${LEMMY_HOSTNAME:=localhost}"
    : "${BIND_ADDR:=0.0.0.0}"
    : "${PORT:=8536}"
    : "${JWT_SECRET:=changeme}"
    : "${FRONT_END_DIR:=/app/dist}"

    # Default rate_limit settings
    : "${RATE_LIMIT_MESSAGE:=180}"
    : "${RATE_LIMIT_MESSAGE_PER_SECOND:=60}"
    : "${RATE_LIMIT_POST:=6}"
    : "${RATE_LIMIT_POST_PER_SECOND:=600}"
    : "${RATE_LIMIT_REGISTER:=3}"
    : "${RATE_LIMIT_REGISTER_PER_SECOND:=3600}"

    # Default federation settings
    : "${FEDERATION_ENABLED:=false}"
    : "${FEDERATION_TLS_ENABLED:=true}"
    : "${FEDERATION_ALLOWED_INSTANCES:=}"

    cat << EOF > /config/config.hjson
{
  database: {
    user: "${DB_USER}"
    password: "${DB_PASSWORD}"
    host: "${DB_HOST}"
    port: ${DB_PORT}
    database: "${DB_DATABASE}"
    pool_size: ${DB_POOL_SIZE}
  }
  hostname: "${LEMMY_HOSTNAME}"
  bind: "${BIND_ADDR}"
  port: ${PORT}
  jwt_secret: "${JWT_SECRET}"
  front_end_dir: "${FRONT_END_DIR}"
  rate_limit: {
    message: ${RATE_LIMIT_MESSAGE}
    message_per_second: ${RATE_LIMIT_MESSAGE_PER_SECOND}
    post: ${RATE_LIMIT_POST}
    post_per_second: ${RATE_LIMIT_POST_PER_SECOND}
    register: ${RATE_LIMIT_REGISTER}
    register_per_second: ${RATE_LIMIT_REGISTER_PER_SECOND}
  }
  federation: {
    enabled: ${FEDERATION_ENABLED}
    tls_enabled: ${FEDERATION_TLS_ENABLED}
    allowed_instances: "${FEDERATION_ALLOWED_INSTANCES}"
  }
}
EOF
fi

exec "$@"

