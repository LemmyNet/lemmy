#!/bin/sh
set -e

sudo chown -R 991:991 volumes/pictrs
sudo docker build ../../ --file ../dev/Dockerfile -t lemmy-dev:latest
sudo docker-compose up -d
