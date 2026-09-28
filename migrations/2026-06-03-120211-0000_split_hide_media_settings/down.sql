ALTER TABLE local_user RENAME COLUMN hide_posts_with_media TO hide_media;

ALTER TABLE local_user
    DROP COLUMN show_media;

