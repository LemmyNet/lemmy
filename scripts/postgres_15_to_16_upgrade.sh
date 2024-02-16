#!/bin/sh
set -e

echo "Do not stop in the middle of this upgrade, wait until you see the message: Upgrade complete."

echo "Stopping lemmy and all services..."
sudo docker-compose stop

echo "Make sure postgres is started..."
sudo docker-compose up -d postgres
echo "Waiting..."
sleep 20s

echo "Exporting the Database to 15_16.dump.sql ..."
sudo docker-compose exec -T postgres pg_dumpall -c -U lemmy > 15_16_dump.sql
echo "Done."

echo "Stopping postgres..."
sudo docker-compose stop postgres
echo "Waiting..."
sleep 20s

echo "Removing the old postgres folder"
sudo rm -rf volumes/postgres

echo "Updating docker-compose to use postgres version 16."
sed -i "s/image: postgres:.*/image: postgres:16-alpine/" ./docker-compose.yml

echo "Starting up new postgres..."
sudo docker-compose up -d postgres
echo "Waiting..."
sleep 20s

echo "Importing the database...."
cat 15_16_dump.sql | sudo docker-compose exec -T postgres psql -U lemmy
echo "Done."

echo "Starting up lemmy..."
sudo docker-compose up -d

echo "A copy of your old database is at 15_16.dump.sql . You can delete this file if the upgrade went smoothly."
echo "Upgrade complete."
