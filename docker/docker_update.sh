#!/bin/sh
set -e

mkdir -p volumes/pictrs
sudo chown -R 991:991 volumes/pictrs
sudo docker compose up -d --build
