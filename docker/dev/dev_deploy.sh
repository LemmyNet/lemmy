#!/bin/sh

# Building from the dev branch for dev servers
git checkout dev

# Rebuilding dev docker
docker-compose build
docker tag dev_lemmy:latest dessalines/lemmy:dev
docker push dessalines/lemmy:dev
