#!/bin/bash
set -e

sudo docker build ../../ --file ../dev/Dockerfile -t lemmy-federation:latest

for Item in alpha beta gamma delta epsilon ; do
  sudo mkdir -p volumes/pictrs_$Item
  sudo chown -R 991:991 volumes/pictrs_$Item
done

sudo docker-compose up
