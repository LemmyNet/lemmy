#!/bin/bash
set -e

pushd ../../ui/ || exit
yarn build
popd || exit

pushd ../../server/ || exit
cargo build
popd || exit

sudo docker build ../../ -f Dockerfile -t lemmy-federation:latest

sudo docker-compose up