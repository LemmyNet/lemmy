#!/bin/bash
set -e

# already start rust build in the background
pushd ../../server/ || exit
cargo build &
popd || exit

if [ "$1" = "-yarn" ]; then
  pushd ../../ui/ || exit
  yarn
  yarn build
  popd || exit
fi

# wait for rust build to finish
pushd ../../server/ || exit
cargo build
popd || exit

sudo docker build ../../ --file Dockerfile -t lemmy-federation:latest

sudo docker-compose up
