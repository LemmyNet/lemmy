#!/bin/bash
set -xe

pushd ../../ui
echo "Waiting for Lemmy to start..."
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8540/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8550/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8560/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8570/api/v1/site')" != "200" ]]; do sleep 1; done
while [[ "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8580/api/v1/site')" != "200" ]]; do sleep 1; done
yarn api-test || true
popd
