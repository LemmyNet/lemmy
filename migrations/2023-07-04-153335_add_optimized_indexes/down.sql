-- Drop the new indexes
DROP INDEX idx_person_admin;

DROP INDEX idx_post_aggregates_featured_local_score;

DROP INDEX idx_post_aggregates_featured_local_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_local_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_featured_local_hot;

DROP INDEX idx_post_aggregates_featured_local_active;

DROP INDEX idx_post_aggregates_featured_local_published;

DROP INDEX idx_post_aggregates_published;

DROP INDEX idx_post_aggregates_featured_community_score;

DROP INDEX idx_post_aggregates_featured_community_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_community_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_featured_community_hot;

DROP INDEX idx_post_aggregates_featured_community_active;

DROP INDEX idx_post_aggregates_featured_community_published;

-- Create single column indexes again
CREATE INDEX idx_post_aggregates_score ON post_aggregates (score DESC);

CREATE INDEX idx_post_aggregates_published ON post_aggregates (published DESC);

CREATE INDEX idx_post_aggregates_newest_comment_time ON post_aggregates (newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_newest_comment_time_necro ON post_aggregates (newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_featured_community ON post_aggregates (featured_community DESC);

CREATE INDEX idx_post_aggregates_featured_local ON post_aggregates (featured_local DESC);

CREATE INDEX idx_post_aggregates_hot ON post_aggregates (hot_rank DESC);

CREATE INDEX idx_post_aggregates_active ON post_aggregates (hot_rank_active DESC);

