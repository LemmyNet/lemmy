#!/usr/bin/env bash

psql -U lemmy -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
cat docker/lemmy_dump_2021-01-29_16_13_40.sqldump | psql -U lemmy
psql -U lemmy -c "alter user lemmy with password 'password'"
