# Run with `source`

export PGDATA="$PWD/dev_pgdata"
export LEMMY_DATABASE_URL="postgresql:///lemmy?host=$PWD"

rm -rf $PGDATA
initdb --username=lemmy --auth=trust

postgres -c listen_addresses='' -c "unix_socket_directories='$PWD'"
