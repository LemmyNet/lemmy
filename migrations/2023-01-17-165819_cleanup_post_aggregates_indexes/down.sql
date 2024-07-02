-- Drop the new indexes
DROP INDEX idx_post_aggregates_featured_local_newest_comment_time, idx_post_aggregates_featured_community_newest_comment_time, idx_post_aggregates_featured_local_comments, idx_post_aggregates_featured_community_comments, idx_post_aggregates_featured_local_hot, idx_post_aggregates_featured_community_hot, idx_post_aggregates_featured_local_active, idx_post_aggregates_featured_community_active, idx_post_aggregates_featured_local_score, idx_post_aggregates_featured_community_score, idx_post_aggregates_featured_local_published, idx_post_aggregates_featured_community_published;

-- Create the old indexes
CREATE INDEX idx_post_aggregates_newest_comment_time ON post_aggregates (newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_comments ON post_aggregates (comments DESC);

CREATE INDEX idx_post_aggregates_hot ON post_aggregates (hot_rank (score, published) DESC, published DESC);

CREATE INDEX idx_post_aggregates_active ON post_aggregates (hot_rank (score, newest_comment_time_necro) DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_score ON post_aggregates (score DESC);

CREATE INDEX idx_post_aggregates_published ON post_aggregates (published DESC);

