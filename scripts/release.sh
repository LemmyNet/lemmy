#!/bin/sh
set -e
#git checkout main

# Creating the new tag
new_tag="$1"
third_semver=$(echo $new_tag | cut -d "." -f 3)

# Goto the upper route
CWD="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
cd "$CWD/../"

# The docker installs should only update for non release-candidates
# IE, when the third semver is a number, not '2-rc'
if [ ! -z "${third_semver##*[!0-9]*}" ]; then
  pushd docker
  sed -i "s/dessalines\/lemmy:.*/dessalines\/lemmy:$new_tag/" docker-compose.yml
  sed -i "s/dessalines\/lemmy-ui:.*/dessalines\/lemmy-ui:$new_tag/" docker-compose.yml
  sed -i "s/dessalines\/lemmy-ui:.*/dessalines\/lemmy-ui:$new_tag/" federation/docker-compose.yml
  git add docker-compose.yml
  git add federation/docker-compose.yml
  popd
fi

# Update crate versions
old_tag=$(grep version Cargo.toml | head -1 | cut -d'"' -f 2)
sed -i "s/{ version = \"=$old_tag\", path/{ version = \"=$new_tag\", path/g" Cargo.toml
sed -i "s/version = \"$old_tag\"/version = \"$new_tag\"/g" Cargo.toml
git add Cargo.toml
cargo check
git add Cargo.lock

# Update the submodules
git submodule update --remote
git add crates/utils/translations

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
