# This script is meant to be run with `source` so it can set environment variables.

export PGDATA="$PWD/dev_pgdata"
export PGHOST=$PWD
export DATABASE_URL="postgresql://lemmy:password@/lemmy?host=$PWD"
export LEMMY_DATABASE_URL=$DATABASE_URL

# If cluster exists, stop the server and delete the cluster
if [[ -d $PGDATA ]]
then
  # Only stop server if it is running
  pg_status_exit_code=0
  (pg_ctl status > /dev/null) || pg_status_exit_code=$?
  if [[ ${pg_status_exit_code} -ne 3 ]]
  then
    pg_ctl stop --silent
  fi

  rm -rf $PGDATA
fi

config_args=(
  # Only listen to socket in current directory
  -c listen_addresses=
  -c unix_socket_directories=$PWD

  # Write logs to a file in $PGDATA/log
  -c logging_collector=on

  # Allow auto_explain to be turned on
  -c session_preload_libraries=auto_explain

  # Include actual row amounts and run times for query plan nodes
  -c auto_explain.log_analyze=on

  # Don't log parameter values
  -c auto_explain.log_parameter_max_length=0
)

# Create cluster
pg_ctl init --silent --options="--username=postgres --auth=trust --no-instructions"

# Start server
pg_ctl start --silent --options="${config_args[*]}"

# Setup database
psql --quiet -c "CREATE USER lemmy WITH PASSWORD 'password' SUPERUSER;" -U postgres
psql --quiet -c "CREATE DATABASE lemmy WITH OWNER lemmy;" -U postgres
