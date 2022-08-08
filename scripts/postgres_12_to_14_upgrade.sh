#!/bin/sh
set -e

echo "Do not stop in the middle of this upgrade, wait until you see the message: Upgrade complete."

echo "Make sure postgres is started..."
sudo docker-compose up -d postgres
sleep 20s

echo "Exporting the Database to 12_14.dump.sql ..."
sudo docker-compose exec -T postgres pg_dumpall -c -U lemmy > 12_14_dump.sql
echo "Done."

echo "Stopping postgres..."
sudo docker-compose stop postgres
sleep 20s

echo "Removing the old postgres folder"
sudo rm -rf volumes/postgres

echo "Updating docker-compose to use postgres version 14."
sed -i "s/postgres:12-alpine/postgres:14-alpine/" ./docker-compose.yml

echo "Starting up new postgres..."
sudo docker-compose up -d postgres
sleep 20s

echo "Importing the database...."
cat 12_14_dump.sql | sudo docker-compose exec -T postgres psql -U lemmy
echo "Done."

POSTGRES_PASSWORD=$(grep "POSTGRES_PASSWORD" ./docker-compose.yml | cut -d"=" -f2)

echo "Fixing a weird password issue with postgres 14"
sudo docker-compose exec -T postgres psql -U lemmy -c "alter user lemmy with password '$POSTGRES_PASSWORD'"
sudo docker-compose restart postgres

echo "Setting correct perms for pictrs folder"
sudo chown -R 991:991 volumes/pictrs

echo "Starting up lemmy..."
sudo docker-compose up -d

echo "A copy of your old database is at 12_14.dump.sql . You can delete this file if the upgrade went smoothly."
echo "Upgrade complete."
