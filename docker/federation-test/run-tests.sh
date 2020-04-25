#!/bin/bash
set -e

pushd ../../server/
cargo build
popd

sudo docker build ../../ --file ../federation/Dockerfile --tag lemmy-federation:latest

sudo docker-compose --file ../federation/docker-compose.yml --project-directory . up -d

pushd ../../ui
yarn
echo "Waiting for Lemmy to start..."
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8540/api/v1/site')" != "200" ]]; do sleep 5; done
yarn api-test || true
popd

sudo docker-compose --file ../federation/docker-compose.yml --project-directory . down

sudo rm -r volumes/
