ALTER TABLE local_site
    ADD COLUMN content_warning text;

ALTER TABLE community
    ADD COLUMN only_followers_can_vote boolean NOT NULL DEFAULT FALSE;

