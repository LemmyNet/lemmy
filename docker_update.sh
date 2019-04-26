#!/bin/sh
set -e

git pull
docker-compose up -d --no-deps --build
