# This script is meant to be run with `source` so it can set environment variables.

export PGDATA="$PWD/dev_pgdata"
export PGHOST=$PWD
export LEMMY_DATABASE_URL="postgresql://lemmy:password@/lemmy?host=$PWD"

# If cluster exists, stop the server and delete the cluster
if [[ -d $PGDATA ]]
then
  # Only stop server if it is running
  (pg_ctl status > /dev/null) || pg_status_exit_code=$?
  if [[ ${pg_status_exit_code} -ne 3 ]]
  then
    pg_ctl stop
  fi

  rm -rf $PGDATA
fi

config_args=(
  # Only listen to socket in current directory
  -c listen_addresses=
  -c unix_socket_directories=$PWD

  # Write logs to a file in $PGDATA/log
  -c logging_collector=on

  # Log all query plans by default
  -c session_preload_libraries=auto_explain
  -c auto_explain.log_min_duration=0

  # Include actual row amounts and run times for query plan nodes
  -c auto_explain.log_analyze=on

  # Avoid sequential scans so query plans show what index scans can be done
  # (index scan is normally avoided in some cases, such as the table being small enough)
  -c enable_seqscan=off

  # Don't log parameter values
  -c auto_explain.log_parameter_max_length=0
)

# Create cluster
initdb --username=postgres --auth=trust --no-instructions

# Start server that only listens to socket in current directory
pg_ctl start --options="${config_args[*]}"

# Setup database
psql -c "CREATE USER lemmy WITH PASSWORD 'password' SUPERUSER;" -U postgres
psql -c "CREATE DATABASE lemmy WITH OWNER lemmy;" -U postgres
