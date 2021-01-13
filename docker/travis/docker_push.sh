#!/bin/sh
echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_USERNAME" --password-stdin
docker tag dessalines/lemmy:travis \
  dessalines/lemmy:0.9.0-rc.4
docker push dessalines/lemmy:0.9.0-rc.4
