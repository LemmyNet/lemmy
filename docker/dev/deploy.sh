#!/bin/sh
git checkout master

# Creating the new tag
new_tag="$1"
git tag $new_tag

third_semver=$(echo $new_tag | cut -d "." -f 3)

# Setting the version on the front end
cd ../../
echo "export let version: string = '$(git describe --tags)';" > "ui/src/version.ts"
git add "ui/src/version.ts"
# Setting the version on the backend
echo "pub const VERSION: &str = \"$(git describe --tags)\";" > "server/src/version.rs"
git add "server/src/version.rs"

cd docker/dev

# Changing the docker-compose prod
sed -i "s/dessalines\/lemmy:.*/dessalines\/lemmy:$new_tag/" ../prod/docker-compose.yml
git add ../prod/docker-compose.yml

# The commit
git commit -m"Version $new_tag"

# Registering qemu binaries
docker run --rm --privileged multiarch/qemu-user-static:register --reset

# Rebuilding docker
docker-compose build
docker tag dev_lemmy:latest dessalines/lemmy:x64-$new_tag
docker push dessalines/lemmy:x64-$new_tag

# Build for Raspberry Pi / other archs

# Arm currently not working
# docker build -t lemmy:armv7hf -f Dockerfile.armv7hf ../../
# docker tag lemmy:armv7hf dessalines/lemmy:armv7hf-$new_tag
# docker push dessalines/lemmy:armv7hf-$new_tag

# aarch64
# Only do this on major releases (IE the third semver is 0)
if [ $third_semver -eq 0 ]; then
  docker build -t lemmy:aarch64 -f Dockerfile.aarch64 ../../
  docker tag lemmy:aarch64 dessalines/lemmy:arm64-$new_tag
  docker push dessalines/lemmy:arm64-$new_tag
fi

# Creating the manifest for the multi-arch build
if [ $third_semver -eq 0 ]; then
  docker manifest create dessalines/lemmy:$new_tag \
  dessalines/lemmy:x64-$new_tag \
  dessalines/lemmy:arm64-$new_tag
else
  docker manifest create dessalines/lemmy:$new_tag \
  dessalines/lemmy:x64-$new_tag
fi

docker manifest push dessalines/lemmy:$new_tag

# Push
git push origin $new_tag
git push

# Pushing to any ansible deploys
cd ../../ansible
ansible-playbook lemmy.yml --become
