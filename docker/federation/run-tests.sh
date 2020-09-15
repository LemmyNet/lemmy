#!/bin/bash
set -e

# make sure there are no old containers or old data around
sudo docker-compose down
sudo rm -rf volumes

mkdir -p volumes/pictrs_{alpha,beta,gamma,delta,epsilon}
sudo chown -R 991:991 volumes/pictrs_{alpha,beta,gamma,delta,epsilon}

sudo docker build ../../ --file ../dev/Dockerfile --tag lemmy-federation:latest

sudo mkdir -p volumes/pictrs_alpha
sudo chown -R 991:991 volumes/pictrs_alpha

sudo docker-compose up -d

pushd ../../api_tests
echo "Waiting for Lemmy to start..."
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8541/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8551/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8561/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8571/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8581/api/v1/site')" != "200" ]]; do sleep 1; done
yarn
yarn api-test || true
popd

sudo docker-compose down

sudo rm -r volumes
