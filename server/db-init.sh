#!/bin/bash

username=lemmy
dbname=lemmy
port=5432

password=""
password_confirm=""
password_valid=0

while [ "$password_valid" == 0 ]
do
  read -p "Enter database password: " -s password
  echo

  read -p "Verify database password: " -s password_confirm
  echo
  echo

  # Start the loop from the top if either check fails
  if [ -z "$password" ]
  then
    echo "Error: Password cannot be empty." 1>&2
    echo
    continue
  fi
  if [ "$password" != "$password_confirm" ]
  then
    echo "Error: Passwords don't match." 1>&2
    echo
    continue
  fi

  # Set the password_valid variable to break out of the loop
  password_valid=1
done


psql -c "CREATE USER $username WITH PASSWORD '$password' SUPERUSER;" -U postgres
psql -c 'CREATE DATABASE $dbname WITH OWNER $username;' -U postgres
export LEMMY_DATABASE_URL=postgres://$username:$password@localhost:$port/$dbname

echo $LEMMY_DATABASE_URL
