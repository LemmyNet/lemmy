ALTER TABLE post
    ADD COLUMN video_width integer CHECK (video_width > 0),
    ADD COLUMN video_height integer CHECK (video_height > 0);

