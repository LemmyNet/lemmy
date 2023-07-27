ALTER TABLE post
    DROP COLUMN embed_url;

ALTER TABLE post
    ADD COLUMN embed_video_url text;

