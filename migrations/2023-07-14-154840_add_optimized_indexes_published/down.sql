-- Drop the new indexes
DROP INDEX idx_post_aggregates_featured_local_most_comments;

DROP INDEX idx_post_aggregates_featured_local_hot;

DROP INDEX idx_post_aggregates_featured_local_active;

DROP INDEX idx_post_aggregates_featured_local_score;

DROP INDEX idx_post_aggregates_featured_community_hot;

DROP INDEX idx_post_aggregates_featured_community_active;

DROP INDEX idx_post_aggregates_featured_community_score;

DROP INDEX idx_post_aggregates_featured_community_most_comments;

DROP INDEX idx_comment_aggregates_hot;

DROP INDEX idx_comment_aggregates_score;

-- Add the old ones back in
-- featured_local
CREATE INDEX idx_post_aggregates_featured_local_hot ON post_aggregates (featured_local DESC, hot_rank DESC);

CREATE INDEX idx_post_aggregates_featured_local_active ON post_aggregates (featured_local DESC, hot_rank_active DESC);

CREATE INDEX idx_post_aggregates_featured_local_score ON post_aggregates (featured_local DESC, score DESC);

-- featured_community
CREATE INDEX idx_post_aggregates_featured_community_hot ON post_aggregates (featured_community DESC, hot_rank DESC);

CREATE INDEX idx_post_aggregates_featured_community_active ON post_aggregates (featured_community DESC, hot_rank_active DESC);

CREATE INDEX idx_post_aggregates_featured_community_score ON post_aggregates (featured_community DESC, score DESC);

CREATE INDEX idx_comment_aggregates_hot ON comment_aggregates (hot_rank DESC);

CREATE INDEX idx_comment_aggregates_score ON comment_aggregates (score DESC);

