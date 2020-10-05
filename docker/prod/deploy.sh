#!/bin/sh
set -e
#git checkout main

# Creating the new tag
new_tag="$1"
#third_semver=$(echo $new_tag | cut -d "." -f 3)

# Setting the version on the front end
cd ../../
# Setting the version on the backend
echo "pub const VERSION: &str = \"$new_tag\";" > "lemmy_api/src/version.rs"
git add "lemmy_api/src/version.rs"
# Setting the version for Ansible
echo $new_tag > "ansible/VERSION"
git add "ansible/VERSION"

cd docker/prod || exit

# Changing various references to the Lemmy version
sed -i "s/dessalines\/lemmy-ui:.*/dessalines\/lemmy-ui:$new_tag/" ../dev/docker-compose.yml
sed -i "s/dessalines\/lemmy-ui:.*/dessalines\/lemmy-ui:$new_tag/" ../federation/docker-compose.yml
sed -i "s/dessalines\/lemmy:.*/dessalines\/lemmy:$new_tag/" ../prod/docker-compose.yml
sed -i "s/dessalines\/lemmy-ui:.*/dessalines\/lemmy-ui:$new_tag/" ../prod/docker-compose.yml
sed -i "s/dessalines\/lemmy:v.*/dessalines\/lemmy:$new_tag/" ../travis/docker_push.sh

git add ../dev/docker-compose.yml
git add ../federation/docker-compose.yml
git add ../prod/docker-compose.yml
git add ../travis/docker_push.sh

# The commit
git commit -m"Version $new_tag"
git tag $new_tag

# Now doing the building on travis, but leave this in for when you need to do an arm build

# export COMPOSE_DOCKER_CLI_BUILD=1
# export DOCKER_BUILDKIT=1

# # Rebuilding docker
# if [ $third_semver -eq 0 ]; then
#   # TODO get linux/arm/v7 build working
#   # Build for Raspberry Pi / other archs too
#   docker buildx build --platform linux/amd64,linux/arm64 ../../ \
#     --file Dockerfile \
#     --tag dessalines/lemmy:$new_tag \
#     --push
# else
#   docker buildx build --platform linux/amd64 ../../ \
#     --file Dockerfile \
#     --tag dessalines/lemmy:$new_tag \
#     --push
# fi

# Push
git push origin $new_tag
git push

# Pushing to any ansible deploys
# cd ../../../lemmy-ansible || exit
# ansible-playbook -i prod playbooks/site.yml --vault-password-file vault_pass
