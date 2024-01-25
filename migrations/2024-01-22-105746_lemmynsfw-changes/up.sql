ALTER TABLE site
    ADD COLUMN content_warning text;

ALTER TABLE local_site
    ADD COLUMN default_post_listing_mode post_listing_mode_enum NOT NULL DEFAULT 'List';

