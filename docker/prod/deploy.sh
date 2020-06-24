#!/bin/sh
set -e
git checkout master

# Import translations
git fetch weblate
git merge weblate/master

# Creating the new tag
new_tag="$1"
third_semver=$(echo $new_tag | cut -d "." -f 3)

# Setting the version on the front end
cd ../../
echo "export const version: string = '$new_tag';" > "ui/src/version.ts"
git add "ui/src/version.ts"
# Setting the version on the backend
echo "pub const VERSION: &str = \"$new_tag\";" > "server/src/version.rs"
git add "server/src/version.rs"
# Setting the version for Ansible
echo $new_tag > "ansible/VERSION"
git add "ansible/VERSION"

cd docker/prod || exit

# Changing the docker-compose prod
sed -i "s/dessalines\/lemmy:.*/dessalines\/lemmy:$new_tag/" ../prod/docker-compose.yml
sed -i "s/dessalines\/lemmy:.*/dessalines\/lemmy:$new_tag/" ../../ansible/templates/docker-compose.yml
git add ../prod/docker-compose.yml
git add ../../ansible/templates/docker-compose.yml

# The commit
git commit -m"Version $new_tag"
git tag $new_tag

export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_BUILDKIT=1

# Rebuilding docker
if [ $third_semver -eq 0 ]; then
  # TODO get linux/arm/v7 build working
  # Build for Raspberry Pi / other archs too
  docker buildx build --platform linux/amd64,linux/arm64 ../../ \
    --file Dockerfile \
    --tag dessalines/lemmy:$new_tag \
    --push
else
  docker buildx build --platform linux/amd64 ../../ \
    --file Dockerfile \
    --tag dessalines/lemmy:$new_tag \
    --push
fi

# Push
git push origin $new_tag
git push

# Pushing to any ansible deploys
cd ../../../lemmy-ansible || exit
ansible-playbook -i prod playbooks/site.yml --vault-password-file vault_pass
