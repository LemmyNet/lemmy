#!/bin/bash

username=lemmy
dbname=lemmy
port=5432

read -p "Enter database password: " -s password
echo

psql -c "CREATE USER $username WITH PASSWORD '$password' SUPERUSER;" -U postgres
psql -c 'CREATE DATABASE $dbname WITH OWNER $username;' -U postgres
export LEMMY_DATABASE_URL=postgres://$username:$password@localhost:$port/$dbname
