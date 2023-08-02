-- This should use the newest_comment_time_necro, not the newest_comment_time for the hot_rank
DROP INDEX idx_post_aggregates_featured_local_active, idx_post_aggregates_featured_community_active;

CREATE INDEX idx_post_aggregates_featured_local_active ON post_aggregates (featured_local DESC, hot_rank (score, newest_comment_time_necro) DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_featured_community_active ON post_aggregates (featured_community DESC, hot_rank (score, newest_comment_time_necro) DESC, newest_comment_time_necro DESC);

