#!/bin/bash

# SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
#
# SPDX-License-Identifier: AGPL-3.0-only

pushd dev
docker-compose exec postgres pg_dumpall -c -U lemmy > dump_`date +%Y-%m-%d"_"%H_%M_%S`.sql
popd
