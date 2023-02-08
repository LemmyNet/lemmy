#!/bin/bash
set -e

export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_BUILDKIT=1
new_tag="$1"
# Rebuilding dev docker
docker build ../../ -f . -t "richardarpanet/lemmy:$new_tag"
docker push "richardarpanet/lemmy:$new_tag"

# Run the playbook
# pushd ../../../lemmy-ansible
# ansible-playbook -i test playbooks/site.yml
# popd
