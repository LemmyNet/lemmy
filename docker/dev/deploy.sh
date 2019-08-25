#!/bin/sh
git checkout master

# Creating the new tag
new_tag="$1"
git tag $new_tag

# Setting the version on the front end
pushd ../../ui/
node set_version.js
git add src/version.ts
popd

# Changing the docker-compose prod
sed -i "s/dessalines\/lemmy:.*/dessalines\/lemmy:$new_tag/" ../prod/docker-compose.yml
git add ../prod/docker-compose.yml

# The commit
git commit -m"Upping version."

git push origin $new_tag
git push

# Rebuilding docker
./docker_update.sh
docker tag dev_lemmy:latest dessalines/lemmy:$new_tag
docker push dessalines/lemmy:$new_tag
