#!/bin/bash

# SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
#
# SPDX-License-Identifier: AGPL-3.0-only

set -e

sudo docker build ../../ --file ../dev/volume_mount.dockerfile -t lemmy-federation:latest

sudo mkdir -p volumes/pictrs
sudo chown -R 991:991 volumes/pictrs

#sudo docker-compose pull --ignore-pull-failures || true
sudo docker-compose up
