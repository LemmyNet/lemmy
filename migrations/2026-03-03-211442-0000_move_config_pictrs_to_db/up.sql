-- This moves a few pictrs related settings in the config, to the database
CREATE TYPE image_mode_enum AS enum (
    'None',
    'StoreLinkPreviews',
    'ProxyAllImages'
);

ALTER TABLE local_site
    ADD COLUMN image_mode image_mode_enum NOT NULL DEFAULT 'ProxyAllImages',
    ADD COLUMN image_proxy_bypass_domains text,
    ADD COLUMN image_upload_timeout_seconds int NOT NULL DEFAULT 30,
    ADD COLUMN image_max_thumbnail_size int NOT NULL DEFAULT 512,
    ADD COLUMN image_max_avatar_size int NOT NULL DEFAULT 512,
    ADD COLUMN image_max_banner_size int NOT NULL DEFAULT 1024,
    ADD COLUMN image_max_upload_size int NOT NULL DEFAULT 1024,
    ADD COLUMN image_allow_video_uploads boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN image_upload_disabled boolean NOT NULL DEFAULT FALSE;

