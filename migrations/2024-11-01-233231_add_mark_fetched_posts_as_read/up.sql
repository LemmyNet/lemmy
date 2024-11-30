ALTER TABLE local_user
    ADD COLUMN auto_mark_fetched_posts_as_read boolean DEFAULT FALSE NOT NULL;

