DROP INDEX idx_post_community_active, idx_post_community_controversy, idx_post_community_hot, idx_post_community_most_comments, idx_post_community_newest_comment_time, idx_post_community_newest_comment_time_necro, idx_post_community_scaled, idx_post_community_score;

CREATE INDEX idx_post_community_active ON post (featured_local DESC, hot_rank_active DESC, published_at DESC, id DESC, community_id);

CREATE INDEX idx_post_community_controversy ON post (featured_local DESC, controversy_rank DESC, id DESC, community_id);

CREATE INDEX idx_post_community_hot ON post (featured_local DESC, hot_rank DESC, published_at DESC, id DESC, community_id);

CREATE INDEX idx_post_community_most_comments ON post (featured_local DESC, comments DESC, published_at DESC, id DESC, community_id);

CREATE INDEX idx_post_community_newest_comment_time ON post (featured_local DESC, coalesce(newest_comment_time_at, published_at) DESC, id DESC, community_id);

CREATE INDEX idx_post_community_newest_comment_time_necro ON post (featured_local DESC, coalesce(newest_comment_time_necro_at, published_at) DESC, id DESC, community_id);

CREATE INDEX idx_post_community_scaled ON post (featured_local DESC, scaled_rank DESC, published_at DESC, id DESC, community_id);

CREATE INDEX idx_post_community_score ON post (featured_local DESC, score DESC, published_at DESC, id DESC, community_id);

