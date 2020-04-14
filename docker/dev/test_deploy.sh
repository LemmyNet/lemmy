#!/bin/sh

# Rebuilding dev docker
docker-compose build
docker tag dev_lemmy:latest dessalines/lemmy:test
docker push dessalines/lemmy:test

# Run the playbook
pushd ../../../lemmy-ansible
ansible-playbook -i test playbooks/site.yml --vault-password-file vault_pass
popd
