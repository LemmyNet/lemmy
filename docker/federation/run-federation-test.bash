#!/bin/bash
set -e

if [ "$1" = "-yarn" ]; then
  pushd ../../ui/ || exit
  yarn build
  popd || exit
fi

pushd ../../server/ || exit
cargo build
popd || exit

sudo docker build ../../ -f Dockerfile -t lemmy-federation:latest

sudo docker-compose up