#!/bin/sh
set -e

export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_BUILDKIT=1
sudo chown -R 991:991 volumes/pictrs
sudo docker-compose up -d --no-deps --build
