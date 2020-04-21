#!/bin/bash
set -e

BRANCH=$1

git checkout $BRANCH
cd ../../

# Rebuilding dev docker
sudo docker build . -f "docker/dev/Dockerfile" -t "dessalines/lemmy:$BRANCH"
sudo docker push "dessalines/lemmy:$BRANCH"

# Run the playbook
pushd ../lemmy-ansible
ansible-playbook -i test playbooks/site.yml
popd
