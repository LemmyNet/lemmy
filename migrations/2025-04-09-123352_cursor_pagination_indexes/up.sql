-- Taglines
CREATE INDEX idx_tagline_published_id ON tagline (published DESC, id DESC);

-- Some for the vote views
CREATE INDEX idx_comment_actions_like_score ON comment_actions (comment_id, like_score, person_id)
WHERE
    like_score IS NOT NULL;

CREATE INDEX idx_post_actions_like_score ON post_actions (post_id, like_score, person_id)
WHERE
    like_score IS NOT NULL;

-- Fixing the community sorts for an id tie-breaker
DROP INDEX idx_community_lower_name;

DROP INDEX idx_community_hot;

DROP INDEX idx_community_published;

DROP INDEX idx_community_subscribers;

DROP INDEX idx_community_title;

DROP INDEX idx_community_users_active_month;

CREATE INDEX idx_community_lower_name ON community USING btree (lower((name)::text), id);

CREATE INDEX idx_community_hot ON community USING btree (hot_rank DESC, id);

CREATE INDEX idx_community_published ON community USING btree (published DESC, id);

CREATE INDEX idx_community_subscribers ON community USING btree (subscribers DESC, id);

CREATE INDEX idx_community_title ON community USING btree (title, id);

CREATE INDEX idx_community_users_active_month ON community USING btree (users_active_month DESC, id);

-- Create a few missing ones
CREATE INDEX idx_community_users_active_half_year ON community USING btree (users_active_half_year DESC, id);

CREATE INDEX idx_community_users_active_week ON community USING btree (users_active_week DESC, id);

CREATE INDEX idx_community_users_active_day ON community USING btree (users_active_day DESC, id);

CREATE INDEX idx_community_subscribers_local ON community USING btree (subscribers_local DESC, id);

CREATE INDEX idx_community_comments ON community USING btree (comments DESC, id);

CREATE INDEX idx_community_posts ON community USING btree (posts DESC, id);

