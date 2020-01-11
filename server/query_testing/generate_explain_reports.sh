#!/bin/sh

# Do the views first

echo "explain (analyze, format json) select * from user_view" > explain.sql
psql -qAt -U lemmy -f explain.sql > user_view.json

echo "explain (analyze, format json) select * from post_view where user_id is null order by hot_rank desc" > explain.sql
psql -qAt -U lemmy -f explain.sql > post_view.json

echo "explain (analyze, format json) select * from comment_view where user_id is null" > explain.sql
psql -qAt -U lemmy -f explain.sql > comment_view.json

echo "explain (analyze, format json) select * from community_view where user_id is null order by hot_rank desc" > explain.sql
psql -qAt -U lemmy -f explain.sql > community_view.json

grep "Execution Time" *.json

rm explain.sql
