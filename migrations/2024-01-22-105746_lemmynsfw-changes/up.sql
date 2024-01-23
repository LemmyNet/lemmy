ALTER TABLE local_site
    ADD COLUMN content_warning text;

ALTER TABLE local_site
    ADD COLUMN auto_expand_images boolean NOT NULL DEFAULT FALSE;

ALTER TABLE community
    ADD COLUMN only_followers_can_vote boolean NOT NULL DEFAULT FALSE;

