DROP INDEX idx_post_aggregates_featured_community_active;

DROP INDEX idx_post_aggregates_featured_community_controversy;

DROP INDEX idx_post_aggregates_featured_community_hot;

DROP INDEX idx_post_aggregates_featured_community_scaled;

DROP INDEX idx_post_aggregates_featured_community_most_comments;

DROP INDEX idx_post_aggregates_featured_community_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_community_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_featured_community_published;

DROP INDEX idx_post_aggregates_featured_community_score;

CREATE INDEX idx_post_aggregates_featured_community_active ON post_aggregates (featured_community DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_controversy ON post_aggregates (featured_community DESC, controversy_rank DESC);

CREATE INDEX idx_post_aggregates_featured_community_hot ON post_aggregates (featured_community DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_scaled ON post_aggregates (featured_community DESC, scaled_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_most_comments ON post_aggregates (featured_community DESC, comments DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time ON post_aggregates (featured_community DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time_necro ON post_aggregates (featured_community DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_featured_community_published ON post_aggregates (featured_community DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_score ON post_aggregates (featured_community DESC, score DESC, published DESC);

DROP INDEX idx_post_aggregates_community_active;

DROP INDEX idx_post_aggregates_community_controversy;

DROP INDEX idx_post_aggregates_community_hot;

DROP INDEX idx_post_aggregates_community_scaled;

DROP INDEX idx_post_aggregates_community_most_comments;

DROP INDEX idx_post_aggregates_community_newest_comment_time;

DROP INDEX idx_post_aggregates_community_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_community_published;

DROP INDEX idx_post_aggregates_community_score;

