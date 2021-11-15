#!/bin/bash

# For the dialogs, everything is default except:

# Use 127.0.0.1 for domain
# Use "password" for db password
# Use local email, don't test

docker-compose build
sudo chown -R 991:991 public
docker-compose run --rm web bundle exec rake mastodon:setup
docker-compose up
