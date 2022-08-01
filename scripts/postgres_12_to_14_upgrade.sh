##!/bin/sh
set -e

## This script upgrades the postgres from version 12 to 14

## Make sure everything is started
sudo docker-compose start

# Export the DB
echo "Exporting the Database to 12_14.dump.sql ..."
sudo docker-compose exec -T postgres pg_dumpall -c -U lemmy > 12_14_dump.sql
echo "Done."

# Stop everything
sudo docker-compose stop

sleep 10s

# Delete the folder
echo "Removing the old postgres folder"
sudo rm -rf volumes/postgres

# Change the version in your docker-compose.yml
echo "Updating docker-compose to use postgres version 14."
sed -i "s/postgres:12-alpine/postgres:14-alpine/" ./docker-compose.yml

# Start up postgres
echo "Starting up new postgres..."
sudo docker-compose up -d postgres

# Sleep for a bit so it can start up, build the new folders
sleep 20s

# Import the DB
echo "Importing the database...."
cat 12_14_dump.sql | sudo docker-compose exec -T postgres psql -U lemmy
echo "Done."

POSTGRES_PASSWORD=$(grep "POSTGRES_PASSWORD" ./docker-compose.yml | cut -d"=" -f2)

# Fix weird password issue with postgres 14
echo "Fixing a weird password issue with postgres 14"
sudo docker-compose exec -T postgres psql -U lemmy -c "alter user lemmy with password '$POSTGRES_PASSWORD'"
sudo docker-compose restart postgres

# Just in case
sudo chown -R 991:991 volumes/pictrs

# Start up the rest of lemmy
echo "Starting up lemmy"
sudo docker-compose up -d

# Delete the DB Dump? Probably safe to keep it
echo "A copy of your old database is at 12_14.dump.sql . You can delete this file if the upgrade went smoothly."
