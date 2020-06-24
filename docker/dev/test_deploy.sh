#!/bin/bash
set -e

BRANCH=$1

git checkout $BRANCH

export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_BUILDKIT=1

# Rebuilding dev docker
sudo docker build . -f "docker/dev/Dockerfile" -t "dessalines/lemmy:$BRANCH"
sudo docker push "dessalines/lemmy:$BRANCH"

# Run the playbook
pushd ../lemmy-ansible
ansible-playbook -i test playbooks/site.yml
popd
