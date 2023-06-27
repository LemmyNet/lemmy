# Run with `source`

export PGDATA="$PWD/dev_pgdata"

rm -rf $PGDATA
initdb --username=lemmy --auth=trust
