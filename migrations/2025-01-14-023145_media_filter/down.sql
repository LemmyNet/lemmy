ALTER TABLE local_user
    DROP COLUMN hide_media;

DROP INDEX idx_post_url_content_type;

