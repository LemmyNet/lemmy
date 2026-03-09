ALTER TABLE local_site
    DROP COLUMN image_mode,
    DROP COLUMN image_proxy_bypass_domains,
    DROP COLUMN image_upload_timeout_seconds,
    DROP COLUMN image_max_thumbnail_size,
    DROP COLUMN image_max_avatar_size,
    DROP COLUMN image_max_banner_size,
    DROP COLUMN image_max_upload_size,
    DROP COLUMN image_allow_video_uploads,
    DROP COLUMN image_upload_disabled;

DROP TYPE image_mode_enum;

