#!/bin/bash
set -e
# This script relies on https://github.com/pksunkara/cargo-workspaces

OLD_VERSION=$(grep version Cargo.toml | head -1 | cut -d '=' -f 2 | cut -d '"' -f 2)
NEW_VERSION=$(git describe --tags --exact-match)

if [ "$OLD_VERSION" == "$NEW_VERSION" ]; then
  echo "Invalid new version"
  exit
fi

ROOT=$(pwd)
for DIR in crates/*; do
  cd $DIR
  pwd
  sed -i "s/{ version = \"$OLD_VERSION\", path/{ version = \"$NEW_VERSION\", path/g" Cargo.toml
  cd $ROOT
done
sed -i "s/{ version = \"$OLD_VERSION\", path/{ version = \"$NEW_VERSION\", path/g" Cargo.toml

cp -r migrations crates/db_queries/
cargo workspaces publish --no-git-commit --allow-dirty --force '*' --skip-published custom "$NEW_VERSION"
rm -r crates/db_queries/migrations/
