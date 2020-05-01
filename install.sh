#!/bin/bash
set -e

# Set the database variable to the default first.
# Don't forget to change this string to your actual database parameters
# if you don't plan to initialize the database in this script.
export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy

# Set other environment variables
export JWT_SECRET=changeme
export HOSTNAME=rrr

# Optionally initialize the database
init_db_valid=0
init_db_final=0
while [ "$init_db_valid" == 0 ]
do
  read -p "Initialize database (y/n)? " init_db
  case "$init_db" in
    [yY]* ) init_db_valid=1; init_db_final=1;;
    [nN]* ) init_db_valid=1; init_db_final=0;;
    * ) echo "Invalid input. Please enter either \"y\" or \"n\"." 1>&2;;
  esac
  echo
done
if [ "$init_db_final" = 1 ]
then
  source ./server/db-init.sh
  read -n 1 -s -r -p "Press ANY KEY to continue execution of this script, press CTRL+C to quit..."
  echo
fi

# Build the web client
cd ui
yarn
yarn build

# Build and run the backend
cd ../server
RUST_LOG=debug cargo run

# For live coding, where both the front and back end, automagically reload on any save, do:
# cd ui && yarn start
# cd server && cargo watch -x run
