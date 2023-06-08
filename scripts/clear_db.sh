#!/usr/bin/env bash

psql -U lemmy -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public; DROP SCHEMA utils CASCADE;"
