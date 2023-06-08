#!/usr/bin/env bash
set -e

# Set the database variable to the default first.
# Don't forget to change this string to your actual database parameters
# if you don't plan to initialize the database in this script.
export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy

# Set other environment variables
export JWT_SECRET=changeme
export HOSTNAME=rrr

yes_no_prompt_invalid() {
  echo "Invalid input. Please enter either \"y\" or \"n\"." 1>&2
}

ask_to_init_db() {
  init_db_valid=0
  init_db_final=0
  while [ "$init_db_valid" == 0 ]
  do
    read -p "Initialize database (y/n)? " init_db
    case "$init_db" in
      [yY]* ) init_db_valid=1; init_db_final=1;;
      [nN]* ) init_db_valid=1; init_db_final=0;;
      * ) yes_no_prompt_invalid;;
    esac
    echo
  done
  if [ "$init_db_final" = 1 ]
  then
    source ./db-init.sh
    read -n 1 -s -r -p "Press ANY KEY to continue execution of this script, press CTRL+C to quit..."
    echo
  fi
}

ask_to_auto_reload() {
  auto_reload_valid=0
  auto_reload_final=0
  while [ "$auto_reload_valid" == 0 ]
  do
    echo "Automagically reload the project when source files are changed?"
    echo "ONLY ENABLE THIS FOR DEVELOPMENT!"
    read -p "(y/n) " auto_reload
    case "$auto_reload" in
      [yY]* ) auto_reload_valid=1; auto_reload_final=1;;
      [nN]* ) auto_reload_valid=1; auto_reload_final=0;;
      * ) yes_no_prompt_invalid;;
    esac
    echo
  done
  if [ "$auto_reload_final" = 1 ]
  then
    cd ui && yarn start
    cd server && cargo watch -x run
  fi
}

# Optionally initialize the database
ask_to_init_db

# Build the web client
cd ui
yarn
yarn build

# Build and run the backend
cd ../server
RUST_LOG=debug cargo run

# For live coding, where both the front and back end, automagically reload on any save
ask_to_auto_reload
