#!/bin/sh
sudo chmod -R 777 volumes
docker-compose up -d --no-deps --build
