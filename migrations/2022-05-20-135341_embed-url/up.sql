ALTER TABLE post
    DROP COLUMN embed_html;

ALTER TABLE post
    ADD COLUMN embed_video_url text;

