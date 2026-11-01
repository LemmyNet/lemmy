-- This removes featured_local from the beginning of the community type indexes, so that the subscribed and multi-community feeds are fast without showing them.
--
DROP INDEX idx_post_community_active, idx_post_community_controversy, idx_post_community_hot, idx_post_community_most_comments, idx_post_community_newest_comment_time, idx_post_community_newest_comment_time_necro, idx_post_community_scaled, idx_post_community_score;

CREATE INDEX idx_post_community_active ON post (hot_rank_active DESC, published_at DESC, id DESC, community_id);

CREATE INDEX idx_post_community_controversy ON post (controversy_rank DESC, id DESC, community_id);

CREATE INDEX idx_post_community_hot ON post (hot_rank DESC, published_at DESC, id DESC, community_id);

CREATE INDEX idx_post_community_most_comments ON post (comments DESC, published_at DESC, id DESC, community_id);

CREATE INDEX idx_post_community_newest_comment_time ON post (coalesce(newest_comment_time_at, published_at) DESC, id DESC, community_id);

CREATE INDEX idx_post_community_newest_comment_time_necro ON post (coalesce(newest_comment_time_necro_at, published_at) DESC, id DESC, community_id);

CREATE INDEX idx_post_community_scaled ON post (scaled_rank DESC, published_at DESC, id DESC, community_id);

CREATE INDEX idx_post_community_score ON post (score DESC, published_at DESC, id DESC, community_id);

