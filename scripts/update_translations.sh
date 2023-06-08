#!/usr/bin/env bash
set -e

pushd ../../lemmy-translations
git fetch weblate
git merge weblate/main
git push
popd

git submodule update --remote
git add ../crates/utils/translations
git commit -m"Updating translations."
git push
