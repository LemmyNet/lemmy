#!/bin/bash
set -e

sudo docker build ../../ --file ../dev/Dockerfile -t lemmy-federation:latest

sudo mkdir -p volumes/pictrs
sudo chown -R 991:991 volumes/pictrs

sudo docker-compose up
