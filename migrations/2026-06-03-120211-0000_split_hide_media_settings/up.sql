-- Splits hide all media (in the UI), and hide_posts_with_media (IE filter out meme posts from fetch results) as two different settings
-- See https://github.com/LemmyNet/lemmy/issues/6564 for context
ALTER TABLE local_user RENAME COLUMN hide_media TO hide_posts_with_media;

ALTER TABLE local_user
    ADD COLUMN show_media boolean DEFAULT TRUE NOT NULL;

