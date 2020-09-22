#!/bin/bash
set -e

export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_BUILDKIT=1

# Rebuilding dev docker
sudo docker build ../../ -f . -t "dessalines/lemmy:dev"
sudo docker push "dessalines/lemmy:dev"

# Run the playbook
# pushd ../../../lemmy-ansible
# ansible-playbook -i test playbooks/site.yml
# popd
