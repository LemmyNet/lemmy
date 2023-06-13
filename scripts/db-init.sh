#!/usr/bin/env bash
set -e

# Default configurations
username=lemmy
password=password
dbname=lemmy
port=5432

yes_no_prompt_invalid() {
  echo "Invalid input. Please enter either \"y\" or \"n\"." 1>&2
}

print_config() {
  echo "  database name: $dbname"
  echo "  username: $username"
  echo "  password: $password"
  echo "  port: $port"
}

ask_for_db_config() {
  echo "The default database configuration is:"
  print_config
  echo

  default_config_final=0
  default_config_valid=0
  while [ "$default_config_valid" == 0 ]
  do
    read -p "Use this configuration (y/n)? " default_config
    case "$default_config" in
      [yY]* ) default_config_valid=1; default_config_final=1;;
      [nN]* ) default_config_valid=1; default_config_final=0;;
      * ) yes_no_prompt_invalid;;
    esac
    echo
  done

  if [ "$default_config_final" == 0 ]
  then
    config_ok_final=0
    while [ "$config_ok_final" == 0 ]
    do
      read -p "Database name:  " dbname
      read -p "Username:  " username
      read -p "Password: password"
      read -p "Port:  " port
      #echo
      
      #echo "The database configuration is:"
      #print_config
      #echo
      
      config_ok_valid=0
      while [ "$config_ok_valid" == 0 ]
      do
        read -p "Use this configuration (y/n)? " config_ok
        case "$config_ok" in
          [yY]* ) config_ok_valid=1; config_ok_final=1;;
          [nN]* ) config_ok_valid=1; config_ok_final=0;;
          * ) yes_no_prompt_invalid;;
        esac
        echo
      done
    done
  fi
}

ask_for_db_config

psql -c "CREATE USER $username WITH PASSWORD '$password' SUPERUSER;" -U postgres
psql -c "CREATE DATABASE $dbname WITH OWNER $username;" -U postgres
export LEMMY_DATABASE_URL=postgres://$username:$password@localhost:$port/$dbname

echo "The database URL is $LEMMY_DATABASE_URL"

