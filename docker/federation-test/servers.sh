#!/bin/bash
set -e

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

sudo docker-compose --file ../federation/docker-compose.yml --project-directory . up
