#!/bin/sh
set -e
docker-compose -f dev/docker-compose.yml up -d --no-deps --build
