ALTER TABLE captcha_answer
    ADD UNIQUE (uuid),
    DROP CONSTRAINT captcha_answer_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE comment_aggregates
    ADD UNIQUE (comment_id),
    DROP CONSTRAINT comment_aggregates_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_comment_like_person ON comment_like (person_id);

ALTER TABLE comment_like
    ADD UNIQUE (comment_id, person_id),
    DROP CONSTRAINT comment_like_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_comment_saved_person_id ON comment_saved (person_id);

ALTER TABLE comment_saved
    ADD UNIQUE (comment_id, person_id),
    DROP CONSTRAINT comment_saved_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE community_aggregates
    ADD UNIQUE (community_id),
    DROP CONSTRAINT community_aggregates_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_community_block_person ON community_block (person_id);

ALTER TABLE community_block
    ADD UNIQUE (person_id, community_id),
    DROP CONSTRAINT community_block_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_community_follower_person ON community_follower (person_id);

ALTER TABLE community_follower
    ADD UNIQUE (community_id, person_id),
    DROP CONSTRAINT community_follower_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE community_language
    ADD UNIQUE (community_id, language_id),
    DROP CONSTRAINT community_language_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_community_moderator_person ON community_moderator (person_id);

ALTER TABLE community_moderator
    ADD UNIQUE (community_id, person_id),
    DROP CONSTRAINT community_moderator_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE community_person_ban
    ADD UNIQUE (community_id, person_id),
    DROP CONSTRAINT community_person_ban_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE custom_emoji_keyword
    ADD UNIQUE (custom_emoji_id, keyword),
    DROP CONSTRAINT custom_emoji_keyword_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE federation_allowlist
    ADD UNIQUE (instance_id),
    DROP CONSTRAINT federation_allowlist_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE federation_blocklist
    ADD UNIQUE (instance_id),
    DROP CONSTRAINT federation_blocklist_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE federation_queue_state
    ADD UNIQUE (instance_id),
    DROP CONSTRAINT federation_queue_state_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE image_upload
    ADD UNIQUE (pictrs_alias),
    DROP CONSTRAINT image_upload_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE instance_block
    ADD UNIQUE (person_id, instance_id),
    DROP CONSTRAINT instance_block_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE local_site_rate_limit
    ADD UNIQUE (local_site_id),
    DROP CONSTRAINT local_site_rate_limit_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE local_user_language
    ADD UNIQUE (local_user_id, language_id),
    DROP CONSTRAINT local_user_language_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE login_token
    ADD UNIQUE (token),
    DROP CONSTRAINT login_token_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE person_aggregates
    ADD UNIQUE (person_id),
    DROP CONSTRAINT person_aggregates_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE person_ban
    ADD UNIQUE (person_id),
    DROP CONSTRAINT person_ban_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE person_block
    ADD UNIQUE (person_id, target_id),
    DROP CONSTRAINT person_block_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE person_follower
    ADD UNIQUE (follower_id, person_id),
    DROP CONSTRAINT person_follower_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE person_post_aggregates
    ADD UNIQUE (person_id, post_id),
    DROP CONSTRAINT person_post_aggregates_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE post_aggregates
    ADD UNIQUE (post_id),
    DROP CONSTRAINT post_aggregates_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_post_like_person ON post_like (person_id);

ALTER TABLE post_like
    ADD UNIQUE (post_id, person_id),
    DROP CONSTRAINT post_like_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE post_read
    ADD UNIQUE (post_id, person_id),
    DROP CONSTRAINT post_read_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE received_activity
    ADD UNIQUE (ap_id),
    DROP CONSTRAINT received_activity_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_post_saved_person_id ON post_saved (person_id);

ALTER TABLE post_saved
    ADD UNIQUE (post_id, person_id),
    DROP CONSTRAINT post_saved_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE site_aggregates
    DROP CONSTRAINT site_aggregates_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE site_language
    ADD UNIQUE (site_id, language_id),
    DROP CONSTRAINT site_language_pkey,
    ADD COLUMN id serial PRIMARY KEY;

