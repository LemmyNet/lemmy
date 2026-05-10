DROP INDEX idx_post_featured_community_published;

DROP INDEX idx_post_community_published;

DROP INDEX idx_post_community_active;

DROP INDEX idx_post_community_controversy;

DROP INDEX idx_post_community_hot;

DROP INDEX idx_post_community_most_comments;

DROP INDEX idx_post_community_newest_comment_time;

DROP INDEX idx_post_community_newest_comment_time_necro;

DROP INDEX idx_post_community_scaled;

DROP INDEX idx_post_community_score;

DROP INDEX idx_post_featured_community_active;

DROP INDEX idx_post_featured_community_controversy;

DROP INDEX idx_post_featured_community_hot;

DROP INDEX idx_post_featured_community_most_comments;

DROP INDEX idx_post_featured_community_newest_comment_time;

DROP INDEX idx_post_featured_community_newest_comment_time_necr;

DROP INDEX idx_post_featured_community_published_asc;

DROP INDEX idx_post_featured_community_scaled;

DROP INDEX idx_post_featured_community_score;

CREATE INDEX idx_post_featured_community_published ON post (community_id, featured_community DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_community_published ON post (community_id, published_at DESC, id DESC);

CREATE INDEX idx_post_community_active ON post (community_id, featured_local DESC, hot_rank_active DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_community_controversy ON post (community_id, featured_local DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_community_hot ON post (community_id, featured_local DESC, hot_rank DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_community_most_comments ON post (community_id, featured_local DESC, comments DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_community_newest_comment_time ON post (community_id, featured_local DESC, coalesce(newest_comment_time_at, published_at) DESC, id DESC);

CREATE INDEX idx_post_community_newest_comment_time_necro ON post (community_id, featured_local DESC, coalesce(newest_comment_time_necro_at, published_at) DESC, id DESC);

CREATE INDEX idx_post_community_scaled ON post (community_id, featured_local DESC, scaled_rank DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_community_score ON post (community_id, featured_local DESC, score DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_community_active ON post (community_id, featured_community DESC, hot_rank_active DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_community_controversy ON post (community_id, featured_community DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_featured_community_hot ON post (community_id, featured_community DESC, hot_rank DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_community_most_comments ON post (community_id, featured_community DESC, comments DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_community_newest_comment_time ON post (community_id, featured_community DESC, coalesce(newest_comment_time_at, published_at) DESC, id DESC);

CREATE INDEX idx_post_featured_community_newest_comment_time_necr ON post (community_id, featured_community DESC, coalesce(newest_comment_time_necro_at, published_at) DESC, id DESC);

CREATE INDEX idx_post_featured_community_scaled ON post (community_id, featured_community DESC, scaled_rank DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_community_score ON post (community_id, featured_community DESC, score DESC, published_at DESC, id DESC);

