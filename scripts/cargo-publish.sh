#!/bin/bash
set -e
# This script relies on https://github.com/pksunkara/cargo-workspaces

OLD_VERSION=0.11.3-rc.4
NEW_VERSION=0.11.3-rc.5
ROOT=$(pwd)
for DIR in crates/*; do
  cd $DIR
  pwd
  sed -i "s/{ version = \"$OLD_VERSION\", path/{ version = \"$NEW_VERSION\", path/g" Cargo.toml
  cd $ROOT
done
sed -i "s/{ version = \"$OLD_VERSION\", path/{ version = \"$NEW_VERSION\", path/g" Cargo.toml

cp -r migrations crates/db_queries/
cargo workspace publish --no-git-commit --allow-dirty --skip-published custom "$NEW_VERSION"
rm -r crates/db_queries/migrations/