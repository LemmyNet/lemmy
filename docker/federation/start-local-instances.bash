#!/bin/bash
set -e

sudo docker build ../../ --file ../dev/Dockerfile -t lemmy-federation:latest

for Item in alpha beta gamma delta epsilon ; do
  sudo mkdir -p volumes/pictrs_$Item
  sudo chown -R 991:991 volumes/pictrs_$Item
done

sudo docker-compose up -d

echo "Waiting for Lemmy to start..."
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8541/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8551/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8561/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8571/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8581/api/v1/site')" != "200" ]]; do sleep 1; done
echo "All instances started."

sudo docker-compose logs -f
