ALTER TABLE login_token
    ADD COLUMN feed_only bool NOT NULL DEFAULT FALSE;

-- only one feed-only token per user
CREATE UNIQUE INDEX idx_login_token_feed_only_user ON login_token (user_id)
WHERE
    feed_only;

