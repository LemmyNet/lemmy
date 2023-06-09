#!/usr/bin/env bash

psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d "$POSTGRES_DB" <<-ESQL
    CREATE USER lemmy WITH PASSWORD 'password';
    CREATE DATABASE lemmy WITH OWNER lemmy;
    GRANT ALL PRIVILEGES ON DATABASE lemmy TO lemmy;
ESQL
