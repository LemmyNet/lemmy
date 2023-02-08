#!/bin/bash
set -e

export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_BUILDKIT=1

# Rebuilding dev docker
# sudo docker build ../../ -f . -t "richardarpanet/lemmy:dev"
# sudo docker push "richardarpanet/lemmy:dev"
docker build ../../ -f ./Dockerfile -t "richardarpanet/lemmy:dev"
docker push "richardarpanet/lemmy:dev"

# Run the playbook
# pushd ../../../lemmy-ansible
# ansible-playbook -i test playbooks/site.yml
# popd
