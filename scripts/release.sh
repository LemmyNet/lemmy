#!/bin/sh

# SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
#
# SPDX-License-Identifier: AGPL-3.0-only

#git checkout main

pushd ../docker/prod/

# Creating the new tag
new_tag="$1"
third_semver=$(echo $new_tag | cut -d "." -f 3)

# The ansible and docker installs should only update for non release-candidates
# IE, when the third semver is a number, not '2-rc'
if [ ! -z "${third_semver##*[!0-9]*}" ]; then
  sed -i "s/dessalines\/lemmy:.*/dessalines\/lemmy:$new_tag/" ../prod/docker-compose.yml
  sed -i "s/dessalines\/lemmy-ui:.*/dessalines\/lemmy-ui:$new_tag/" ../prod/docker-compose.yml
  sed -i "s/dessalines\/lemmy-ui:.*/dessalines\/lemmy-ui:$new_tag/" ../dev/docker-compose.yml
  sed -i "s/dessalines\/lemmy-ui:.*/dessalines\/lemmy-ui:$new_tag/" ../federation/docker-compose.yml
  git add ../prod/docker-compose.yml
  git add ../dev/docker-compose.yml
  git add ../federation/docker-compose.yml

  # Setting the version for Ansible
  pushd ../../../lemmy-ansible
  echo $new_tag > "VERSION"
  git add "VERSION"
  git commit -m"Updating VERSION"
  git tag $new_tag
  git push origin $new_tag
  git push
  popd
fi

# Update crate versions for crates.io
pushd ../../
old_tag=$(head -3 Cargo.toml | tail -1 | cut -d'"' -f 2)
for crate in crates/*; do
  pushd $crate
  # update version of the crate itself (only first occurence)
  # https://stackoverflow.com/a/9453461
  sed -i "0,/version = \"$old_tag\"/s//version = \"$new_tag\"/g" Cargo.toml
  # update version of lemmy dependencies
  sed -i "s/{ version = \"=$old_tag\", path/{ version = \"=$new_tag\", path/g" Cargo.toml
  git add Cargo.toml
  popd
done
# same as above, for the main cargo.toml
sed -i "s/{ version = \"=$old_tag\", path/{ version = \"=$new_tag\", path/g" Cargo.toml
sed -i "s/version = \"$old_tag\"/version = \"$new_tag\"/g" Cargo.toml
git add Cargo.toml
cargo check
git add Cargo.lock
popd

# The commit
git commit -m"Version $new_tag"
git tag $new_tag

# export COMPOSE_DOCKER_CLI_BUILD=1
# export DOCKER_BUILDKIT=1

# Push
git push origin $new_tag
git push

# Pushing to any ansible deploys
# cd ../../../lemmy-ansible || exit
# ansible-playbook -i prod playbooks/site.yml --vault-password-file vault_pass
