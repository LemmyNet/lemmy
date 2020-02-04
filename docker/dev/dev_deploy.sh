#!/bin/sh

# Building from the dev branch for dev servers
git checkout dev

# Rebuilding dev docker
docker-compose build
docker tag dev_lemmy:latest dessalines/lemmy:dev
docker push dessalines/lemmy:dev

# SSH and pull it
ssh $LEMMY_USER@$LEMMY_HOST "cd ~/git/lemmy/docker/dev && docker pull dessalines/lemmy:dev && docker-compose up -d"
