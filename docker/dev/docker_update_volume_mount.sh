#!/bin/sh
set -e

mkdir -p volumes/pictrs
sudo chown -R 991:991 volumes/pictrs
sudo docker build ../../ --file ../dev/volume_mount.dockerfile -t lemmy-dev:latest
sudo docker-compose up -d
