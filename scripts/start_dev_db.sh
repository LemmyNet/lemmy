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

# Create cluster
initdb --username=postgres --auth=trust --no-instructions

# Start server that only listens to socket in current directory
pg_ctl start --options="-c listen_addresses= -c unix_socket_directories=$PWD -c logging_collector=on -c session_preload_libraries=auto_explain -c auto_explain.log_min_duration=0 -c auto_explain.log_parameter_max_length=0 -c auto_explain.log_analyze=on -c enable_seqscan=off" > /dev/null

# Setup database
psql -c "CREATE USER lemmy WITH PASSWORD 'password' SUPERUSER;" -U postgres
psql -c "CREATE DATABASE lemmy WITH OWNER lemmy;" -U postgres
