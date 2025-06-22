# Lemmy Deployment Guide

This guide explains how to deploy Lemmy using the Docker setup included in this repository.

## Prerequisites

- **Operating System**: Debian 12 or Ubuntu 22.04 (or newer).
- **Hardware**: 2 CPU cores, at least 2 GB RAM.
- **Required Ports**: TCP ports `1236` and `8536` for the web UI and API, and `5433` for PostgreSQL.
- **Software**: Docker >= 20, Docker Compose >= v2, PostgreSQL >= 14 (v16 recommended), Nginx, Postfix.

The `docker-compose.yml` exposes the web ports as shown below:

```yaml
services:
  proxy:
    image: nginx:1-alpine
    ports:
      - "1236:1236"
      - "8536:8536"
```

## Directory Layout

Place configuration and data under the `docker/` directory:

```
docker/
  docker-compose.yml
  lemmy.hjson
  nginx.conf
  volumes/
    postgres/   # PostgreSQL data
    pictrs/     # Pictrs media
```

Persistent volumes are mounted from the `volumes` directory by the compose file:

```yaml
postgres:
  volumes:
    - ./volumes/postgres:/var/lib/postgresql/data:Z
pictrs:
  volumes:
    - ./volumes/pictrs:/mnt:Z
```

## Configuration Steps

1. **Edit `docker/lemmy.hjson`**

   Update your domain, database password and pictrs API key.

   ```hjson
   database: {
     connection: "postgres://lemmy:<password>@postgres:5432/lemmy"
   }

   hostname: "<your-domain>"

   pictrs: {
     url: "http://pictrs:8080/"
     api_key: "<pictrs-key>"
   }
   ```

2. **Nginx Mapping**

   The bundled `nginx.conf` proxies requests to the containers. If you run Nginx outside of Docker, map the service names to Docker’s DNS IP `127.0.0.11` in an `nginx_internal.conf`:

   ```nginx
   upstream lemmy { server 127.0.0.11:8536; }
   upstream lemmy-ui { server 127.0.0.11:1234; }
   ```

3. **Run Docker Compose**

   Start services in the background:

   ```bash
   docker compose -f docker/docker-compose.yml up -d
   ```

   Stop them with:

   ```bash
   docker compose -f docker/docker-compose.yml down
   ```

## PostgreSQL Tuning

The compose file uses the image `pgautoupgrade/pgautoupgrade:16-alpine` which bundles PostgreSQL 16. Older distributions (such as the default PostgreSQL 12 shipped with Ubuntu) are not supported. Adjust your host settings accordingly if running PostgreSQL outside of Docker.

## Optional Enhancements

- **Pictrs**: The image server container already uses `/volumes/pictrs` for storage. Adjust the `PICTRS__MEDIA` settings in `docker-compose.yml` to tune conversions or file sizes.
- **Email**: Configure Postfix so Lemmy can send from `noreply@<hostname>`.
- **Synology NAS**: Ensure the above ports are allowed and set shared folders for the `volumes` directory if running on Synology.

## Validation

After `docker compose up -d`, verify:

1. Open `http://<hostname>:1236` and ensure the Lemmy UI loads.
2. Check that database migrations completed successfully (`docker compose logs postgres`).
3. Upload an image to confirm Pictrs works.
4. Send a test email from Lemmy and confirm it arrives from `noreply@<hostname>`.

To shut down and remove volumes:

```bash
docker compose -f docker/docker-compose.yml down --volumes
```

