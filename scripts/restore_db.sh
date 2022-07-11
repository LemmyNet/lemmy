#!/bin/bash

# SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
#
# SPDX-License-Identifier: AGPL-3.0-only

psql -U lemmy -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
cat docker/dev/lemmy_dump_2021-01-29_16_13_40.sqldump | psql -U lemmy
psql -U lemmy -c "alter user lemmy with password 'password'"
