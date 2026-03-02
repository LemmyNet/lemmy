#!/usr/bin/env bash
set -e

export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_BUILDKIT=1

# Rebuilding dev docker
pushd ..
sudo docker build . -f docker/Dockerfile --build-arg RUST_RELEASE_MODE=release -t "dessalines/lemmy:dev" --platform=linux/amd64 --push

# Run the playbook
# pushd ../../../lemmy-ansible
# ansible-playbook -i test playbooks/site.yml
# popd
