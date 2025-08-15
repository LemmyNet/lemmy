ALTER TABLE local_user
    ADD COLUMN hide_media boolean DEFAULT FALSE NOT NULL;

CREATE INDEX idx_post_url_content_type ON post USING gin (url_content_type gin_trgm_ops);

