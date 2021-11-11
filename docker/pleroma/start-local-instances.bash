#!/bin/bash
set -e

sudo docker build ../../ --file ../dev/volume_mount.dockerfile -t lemmy-federation:latest

sudo mkdir -p volumes/pictrs
sudo chown -R 991:991 volumes/pictrs

#sudo docker-compose pull --ignore-pull-failures || true
sudo docker-compose up
