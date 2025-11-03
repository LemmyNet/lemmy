-- Taglines
CREATE INDEX idx_tagline_published_id ON tagline (published DESC, id DESC);

-- Some for the vote views
CREATE INDEX idx_comment_actions_like_score ON comment_actions (comment_id, vote_is_upvote, person_id)
WHERE
    vote_is_upvote IS NOT NULL;

CREATE INDEX idx_post_actions_like_score ON post_actions (post_id, vote_is_upvote, person_id)
WHERE
    vote_is_upvote IS NOT NULL;

-- Fixing the community sorts for an id tie-breaker
DROP INDEX idx_community_lower_name;

DROP INDEX idx_community_hot;

DROP INDEX idx_community_published;

DROP INDEX idx_community_subscribers;

DROP INDEX idx_community_title;

DROP INDEX idx_community_users_active_month;

CREATE INDEX idx_community_lower_name ON community USING btree (lower((name)::text) DESC, id DESC);

CREATE INDEX idx_community_hot ON community USING btree (hot_rank DESC, id DESC);

CREATE INDEX idx_community_published ON community USING btree (published DESC, id DESC);

CREATE INDEX idx_community_subscribers ON community USING btree (subscribers DESC, id DESC);

CREATE INDEX idx_community_title ON community USING btree (title DESC, id DESC);

CREATE INDEX idx_community_users_active_month ON community USING btree (users_active_month DESC, id DESC);

-- Create a few missing ones
CREATE INDEX idx_community_users_active_half_year ON community USING btree (users_active_half_year DESC, id DESC);

CREATE INDEX idx_community_users_active_week ON community USING btree (users_active_week DESC, id DESC);

CREATE INDEX idx_community_users_active_day ON community USING btree (users_active_day DESC, id DESC);

CREATE INDEX idx_community_subscribers_local ON community USING btree (subscribers_local DESC, id DESC);

CREATE INDEX idx_community_comments ON community USING btree (comments DESC, id DESC);

CREATE INDEX idx_community_posts ON community USING btree (posts DESC, id DESC);

-- Fix the post reverse_timestamp key sorts.
DROP INDEX idx_post_featured_community_published_asc;

DROP INDEX idx_post_featured_local_published_asc;

DROP INDEX idx_post_published_asc;

CREATE INDEX idx_post_featured_community_published ON post USING btree (community_id, featured_community DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_published ON post USING btree (community_id, published DESC, id DESC);

