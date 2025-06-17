ALTER TABLE post
    DROP COLUMN embed_video_url;

ALTER TABLE post
    ADD COLUMN embed_html text;

