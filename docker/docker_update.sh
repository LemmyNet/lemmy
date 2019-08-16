#!/bin/sh
set -e

git pull
docker-compose -f dev/docker-compose.yml up -d --no-deps --build
