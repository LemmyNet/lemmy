#!/bin/sh
echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_USERNAME" --password-stdin
docker tag dessalines/lemmy:travis \
  dessalines/lemmy:v0.7.41
docker push dessalines/lemmy:v0.7.41
