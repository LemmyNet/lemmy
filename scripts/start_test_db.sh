export PGUSER=lemmy
export LEMMY_DATABASE_HOST=localhost
export LEMMY_DATABASE_PORT=5433
export LEMMY_DATABASE_URL="postgresql://lemmy:password@$LEMMY_DATABASE_HOST:$LEMMY_DATABASE_PORT/lemmy"
export PGDATABASE=lemmy

docker-compose -f docker/docker-compose-test.yml up -d postgres
