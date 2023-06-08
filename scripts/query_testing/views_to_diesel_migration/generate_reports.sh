#!/usr/bin/env bash
set -e

# You can import these to http://tatiyants.com/pev/#/plans/new

pushd reports

PSQL_CMD="docker exec -i dev_postgres_1 psql -qAt -U lemmy"

echo "explain (analyze, format json) select * from user_ limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > user_.json

echo "explain (analyze, format json) select * from post p limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > post.json

echo "explain (analyze, format json) select * from post p, post_aggregates pa where p.id = pa.post_id order by hot_rank(pa.score, pa.published) desc, pa.published desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > post_ordered_by_rank.json

echo "explain (analyze, format json) select * from post p, post_aggregates pa where p.id = pa.post_id order by pa.stickied desc, hot_rank(pa.score, pa.published) desc, pa.published desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > post_ordered_by_stickied_then_rank.json

echo "explain (analyze, format json) select * from post p, post_aggregates pa where p.id = pa.post_id order by pa.score desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > post_ordered_by_score.json

echo "explain (analyze, format json) select * from post p, post_aggregates pa where p.id = pa.post_id order by pa.stickied desc, pa.score desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > post_ordered_by_stickied_then_score.json

echo "explain (analyze, format json) select * from post p, post_aggregates pa where p.id = pa.post_id order by pa.published desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > post_ordered_by_published.json

echo "explain (analyze, format json) select * from post p, post_aggregates pa where p.id = pa.post_id order by pa.stickied desc, pa.published desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > post_ordered_by_stickied_then_published.json

echo "explain (analyze, format json) select * from comment limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > comment.json

echo "explain (analyze, format json) select * from community limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > community.json

echo "explain (analyze, format json) select * from community c, community_aggregates ca where c.id = ca.community_id order by hot_rank(ca.subscribers, ca.published) desc, ca.published desc limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > community_ordered_by_subscribers.json

echo "explain (analyze, format json) select * from site s" > explain.sql
cat explain.sql | $PSQL_CMD > site.json

echo "explain (analyze, format json) select * from user_mention limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > user_mention.json

echo "explain (analyze, format json) select * from private_message limit 100" > explain.sql
cat explain.sql | $PSQL_CMD > private_message.json

grep "Execution Time" *.json > ../timings-`date +%Y-%m-%d_%H-%M-%S`.out

rm explain.sql

popd
