#!/usr/bin/env bash
set -e

# You can import these to http://tatiyants.com/pev/#/plans/new

pushd reports

# Do the views first

PSQL_CMD="docker exec -i dev_postgres_1 psql -qAt -U lemmy"

echo "explain (analyze, format json) select * from user_fast limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > user_fast.json

echo "explain (analyze, format json) select * from post_view where user_id is null order by hot_rank desc, published desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > post_view.json

echo "explain (analyze, format json) select * from post_fast_view where user_id is null order by hot_rank desc, published desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > post_fast_view.json

echo "explain (analyze, format json) select * from comment_view where user_id is null limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > comment_view.json

echo "explain (analyze, format json) select * from comment_fast_view where user_id is null limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > comment_fast_view.json

echo "explain (analyze, format json) select * from community_view where user_id is null order by hot_rank desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > community_view.json

echo "explain (analyze, format json) select * from community_fast_view where user_id is null order by hot_rank desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > community_fast_view.json

echo "explain (analyze, format json) select * from site_view limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > site_view.json

echo "explain (analyze, format json) select * from reply_fast_view where user_id = 34 and recipient_id = 34 limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > reply_fast_view.json

echo "explain (analyze, format json) select * from user_mention_view where user_id = 34 and recipient_id = 34 limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > user_mention_view.json

echo "explain (analyze, format json) select * from user_mention_fast_view where user_id = 34 and recipient_id = 34 limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > user_mention_fast_view.json

grep "Execution Time" *.json > ../timings-`date +%Y-%m-%d_%H-%M-%S`.out

rm explain.sql

popd
