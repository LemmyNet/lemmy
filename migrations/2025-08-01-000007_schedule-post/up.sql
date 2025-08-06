ALTER TABLE post
    ADD COLUMN scheduled_publish_time timestamptz;

CREATE INDEX idx_post_scheduled_publish_time ON post (scheduled_publish_time);

