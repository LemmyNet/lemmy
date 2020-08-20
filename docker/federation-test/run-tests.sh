#!/bin/bash
set -e

# make sure there are no old containers or old data around
sudo docker-compose --file ../federation/docker-compose.yml --project-directory . down
sudo rm -rf volumes

pushd ../../server/
cargo build
popd

pushd ../../ui
yarn
popd

mkdir -p volumes/pictrs_{alpha,beta,gamma,delta,epsilon}
sudo chown -R 991:991 volumes/pictrs_{alpha,beta,gamma,delta,epsilon}

sudo docker build ../../ --file ../federation/Dockerfile --tag lemmy-federation:latest

sudo mkdir -p volumes/pictrs_alpha
sudo chown -R 991:991 volumes/pictrs_alpha

sudo docker-compose --file ../federation/docker-compose.yml --project-directory . up -d

pushd ../../ui
echo "Waiting for Lemmy to start..."
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8540/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8550/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8560/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8570/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8580/api/v1/site')" != "200" ]]; do sleep 1; done
yarn api-test || true
popd

sudo docker-compose --file ../federation/docker-compose.yml --project-directory . down

sudo rm -r volumes/
