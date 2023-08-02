-- Create an admin person index
CREATE INDEX IF NOT EXISTS idx_person_admin ON person (admin);

-- Compound indexes, using featured_, then the other sorts, proved to be much faster
-- Drop the old indexes
DROP INDEX idx_post_aggregates_score;

DROP INDEX idx_post_aggregates_published;

DROP INDEX idx_post_aggregates_newest_comment_time;

DROP INDEX idx_post_aggregates_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_featured_community;

DROP INDEX idx_post_aggregates_featured_local;

DROP INDEX idx_post_aggregates_hot;

DROP INDEX idx_post_aggregates_active;

-- featured_local
CREATE INDEX idx_post_aggregates_featured_local_score ON post_aggregates (featured_local DESC, score DESC);

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time ON post_aggregates (featured_local DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time_necro ON post_aggregates (featured_local DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_featured_local_hot ON post_aggregates (featured_local DESC, hot_rank DESC);

CREATE INDEX idx_post_aggregates_featured_local_active ON post_aggregates (featured_local DESC, hot_rank_active DESC);

CREATE INDEX idx_post_aggregates_featured_local_published ON post_aggregates (featured_local DESC, published DESC);

CREATE INDEX idx_post_aggregates_published ON post_aggregates (published DESC);

-- featured_community
CREATE INDEX idx_post_aggregates_featured_community_score ON post_aggregates (featured_community DESC, score DESC);

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time ON post_aggregates (featured_community DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time_necro ON post_aggregates (featured_community DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_featured_community_hot ON post_aggregates (featured_community DESC, hot_rank DESC);

CREATE INDEX idx_post_aggregates_featured_community_active ON post_aggregates (featured_community DESC, hot_rank_active DESC);

CREATE INDEX idx_post_aggregates_featured_community_published ON post_aggregates (featured_community DESC, published DESC);

