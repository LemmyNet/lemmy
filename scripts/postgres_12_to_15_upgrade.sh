#!/bin/sh
set -e

echo "Do not stop in the middle of this upgrade, wait until you see the message: Upgrade complete."

echo "Stopping lemmy and all services..."
sudo docker-compose stop

echo "Make sure postgres is started..."
sudo docker-compose up -d postgres
sleep 20s

echo "Exporting the Database to 12_15.dump.sql ..."
sudo docker-compose exec -T postgres pg_dumpall -c -U lemmy > 12_15_dump.sql
echo "Done."

echo "Stopping postgres..."
sudo docker-compose stop postgres
sleep 20s

echo "Removing the old postgres folder"
sudo rm -rf volumes/postgres

echo "Updating docker-compose to use postgres version 15."
sudo sed -i "s/image: .*postgres:.*/image: pgautoupgrade\/pgautoupgrade:15-alpine/" ./docker-compose.yml

echo "Starting up new postgres..."
sudo docker-compose up -d postgres
sleep 20s

echo "Importing the database...."
cat 12_15_dump.sql | sudo docker-compose exec -T postgres psql -U lemmy
echo "Done."

POSTGRES_PASSWORD=$(grep "POSTGRES_PASSWORD" ./docker-compose.yml | cut -d"=" -f2)

echo "Fixing a weird password issue with postgres 15"
sudo docker-compose exec -T postgres psql -U lemmy -c "alter user lemmy with password '$POSTGRES_PASSWORD'"
sudo docker-compose restart postgres

echo "Setting correct perms for pictrs folder"
sudo chown -R 991:991 volumes/pictrs

echo "Starting up lemmy..."
sudo docker-compose up -d

echo "A copy of your old database is at 12_15.dump.sql . You can delete this file if the upgrade went smoothly."
echo "Upgrade complete."
