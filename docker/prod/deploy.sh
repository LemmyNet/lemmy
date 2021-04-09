#!/bin/sh
set -e
#git checkout main

# Creating the new tag
new_tag="$1"
third_semver=$(echo $new_tag | cut -d "." -f 3)

# Setting the version on the backend
pushd ../../
echo "pub const VERSION: &str = \"$new_tag\";" > "crates/utils/src/version.rs"
git add "crates/utils/src/version.rs"
popd

# The ansible and docker installs should only update for non release-candidates
# IE, when the third semver is a number, not '2-rc'
if [ ! -z "${third_semver##*[!0-9]*}" ]; then
  sed -i "s/dessalines\/lemmy:.*/dessalines\/lemmy:$new_tag/" ../prod/docker-compose.yml
  sed -i "s/dessalines\/lemmy-ui:.*/dessalines\/lemmy-ui:$new_tag/" ../prod/docker-compose.yml
  git add ../prod/docker-compose.yml

  # Setting the version for Ansible
  pushd ../../
  echo $new_tag > "ansible/VERSION"
  git add "ansible/VERSION"
  popd
fi

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
