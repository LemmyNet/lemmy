DROP INDEX idx_tagline_published_id;

DROP INDEX idx_comment_actions_like_score;

DROP INDEX idx_post_actions_like_score;

-- Fixing the community sorts for an id tie-breaker
DROP INDEX idx_community_lower_name;

DROP INDEX idx_community_hot;

DROP INDEX idx_community_published;

DROP INDEX idx_community_subscribers;

DROP INDEX idx_community_title;

DROP INDEX idx_community_users_active_month;

CREATE INDEX idx_community_lower_name ON community USING btree (lower((name)::text));

CREATE INDEX idx_community_hot ON community USING btree (hot_rank DESC);

CREATE INDEX idx_community_published ON community USING btree (published DESC);

CREATE INDEX idx_community_subscribers ON community USING btree (subscribers DESC);

CREATE INDEX idx_community_title ON community USING btree (title);

CREATE INDEX idx_community_users_active_month ON community USING btree (users_active_month DESC);

-- Drop the missing ones.
DROP INDEX idx_community_users_active_half_year;

DROP INDEX idx_community_users_active_week;

DROP INDEX idx_community_users_active_day;

DROP INDEX idx_community_subscribers_local;

DROP INDEX idx_community_comments;

DROP INDEX idx_community_posts;

-- Fix the post reverse_timestamp key sorts.
DROP INDEX idx_post_community_published;

DROP INDEX idx_post_featured_community_published;

CREATE INDEX idx_post_featured_community_published_asc ON post USING btree (community_id, featured_community DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_featured_local_published_asc ON post USING btree (featured_local DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_published_asc ON post USING btree (reverse_timestamp_sort (published) DESC);

