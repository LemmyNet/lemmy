#!/bin/bash
set -e

# already start rust build in the background
pushd ../../server/ || exit
cargo build &
popd || exit

if [ "$1" != "--no-yarn-build" ]; then
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

for Item in alpha beta gamma delta epsilon ; do
  sudo mkdir -p volumes/pictrs_$Item
  sudo chown -R 991:991 volumes/pictrs_$Item
done

sudo docker-compose up
