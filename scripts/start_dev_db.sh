# This script is meant to be run with `source` so it can set environment variables.

export PGDATA="$PWD/dev_pgdata"
export PGHOST=$PWD
export LEMMY_DATABASE_URL="postgresql://lemmy:password@/lemmy?host=$PWD"

# If cluster exists, stop the server and delete the cluster
if [ -d $PGDATA ]
then
  # Prevent `stop` from failing if server already stopped
  pg_ctl restart > /dev/null
  pg_ctl stop
  rm -rf $PGDATA
fi

# Create cluster
initdb --username=postgres --auth=trust --no-instructions

# Start server that only listens to socket in current directory
pg_ctl start --options="-c listen_addresses= -c unix_socket_directories=$PWD" > /dev/null

# Setup database
psql -c "CREATE USER lemmy WITH PASSWORD 'password' SUPERUSER;" -U postgres
psql -c "CREATE DATABASE lemmy WITH OWNER lemmy;" -U postgres
