ALTER TABLE local_user
    ADD COLUMN collapse_bot_comments boolean DEFAULT FALSE NOT NULL;

