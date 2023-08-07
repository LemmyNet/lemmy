DROP INDEX idx_post_aggregates_featured_local_active, idx_post_aggregates_featured_community_active;

CREATE INDEX idx_post_aggregates_featured_local_active ON post_aggregates (featured_local DESC, hot_rank (score, newest_comment_time) DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_community_active ON post_aggregates (featured_community DESC, hot_rank (score, newest_comment_time) DESC, newest_comment_time DESC);

