#!/bin/bash

# SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
#
# SPDX-License-Identifier: AGPL-3.0-only

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
