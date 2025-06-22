#!/usr/bin/env bash
set -euo pipefail

# Determine repository root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Variables can be provided via env or defaults will be used
HOSTNAME="${LEMMY_HOSTNAME:-example.com}"
DB_PASSWORD="${LEMMY_DB_PASSWORD:-password}"
PICTRS_API_KEY="${PICTRS_API_KEY:-$(openssl rand -hex 16)}"

# Install required packages if missing
install_pkg() {
  if ! dpkg -s "$1" >/dev/null 2>&1; then
    apt-get update && apt-get install -y "$@"
  fi
}

if ! command -v docker >/dev/null; then
  install_pkg docker.io
fi
if ! command -v docker compose >/dev/null; then
  install_pkg docker-compose-plugin
fi
if ! command -v psql >/dev/null; then
  install_pkg postgresql-client
fi

# Create volume directories
mkdir -p docker/volumes/postgres docker/volumes/pictrs

# Generate lemmy configuration
cat > docker/lemmy.hjson <<CFG
{
  setup: {
    admin_username: "lemmy"
    admin_password: "lemmylemmy"
    site_name: "lemmy-dev"
  }
  database: {
    connection: "postgres://lemmy:${DB_PASSWORD}@postgres:5432/lemmy"
  }
  hostname: "${HOSTNAME}"
  bind: "0.0.0.0"
  port: 8536
  pictrs: {
    url: "http://pictrs:8080/"
    api_key: "${PICTRS_API_KEY}"
    image_mode: None
  }
}
CFG

# Launch services
cd docker
docker compose up -d

# Wait for the UI to respond
printf "Waiting for Lemmy UI..."
for i in {1..30}; do
  if curl -fs http://localhost:1236 >/dev/null 2>&1; then
    echo " ready" && break
  fi
  printf '.'
  sleep 5
done

exit 0
