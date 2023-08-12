-- these indices are used for single-community filtering and for the followed-communities (front-page) query
-- basically one index per Sort
-- index name is truncated to 63 chars so abbreviate a bit
CREATE INDEX idx_post_aggregates_community_active ON post_aggregates (community_id, featured_local DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_community_controversy ON post_aggregates (community_id, featured_local DESC, controversy_rank DESC);

CREATE INDEX idx_post_aggregates_community_hot ON post_aggregates (community_id, featured_local DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_community_most_comments ON post_aggregates (community_id, featured_local DESC, comments DESC, published DESC);

CREATE INDEX idx_post_aggregates_community_newest_comment_time ON post_aggregates (community_id, featured_local DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_community_newest_comment_time_necro ON post_aggregates (community_id, featured_local DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_community_published ON post_aggregates (community_id, featured_local DESC, published DESC);

CREATE INDEX idx_post_aggregates_community_score ON post_aggregates (community_id, featured_local DESC, score DESC, published DESC);

-- these indices are used for "local" filtering
-- these indices weren't really useful because whenever the query filters by featured_community it also filters by community_id, so prepend that to all these indexes
DROP INDEX idx_post_aggregates_featured_community_active;

DROP INDEX idx_post_aggregates_featured_community_controversy;

DROP INDEX idx_post_aggregates_featured_community_hot;

DROP INDEX idx_post_aggregates_featured_community_most_comments;

DROP INDEX idx_post_aggregates_featured_community_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_community_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_featured_community_published;

DROP INDEX idx_post_aggregates_featured_community_score;

CREATE INDEX idx_post_aggregates_featured_community_active ON post_aggregates (community_id, featured_community DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_controversy ON post_aggregates (community_id, featured_community DESC, controversy_rank DESC);

CREATE INDEX idx_post_aggregates_featured_community_hot ON post_aggregates (community_id, featured_community DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_most_comments ON post_aggregates (community_id, featured_community DESC, comments DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time ON post_aggregates (community_id, featured_community DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time_necro ON post_aggregates (community_id, featured_community DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_featured_community_published ON post_aggregates (community_id, featured_community DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_score ON post_aggregates (community_id, featured_community DESC, score DESC, published DESC);

