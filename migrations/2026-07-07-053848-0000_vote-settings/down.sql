ALTER TABLE community
    DROP COLUMN downvote_mode;

DROP TYPE community_downvote_enum;

ALTER TYPE local_site_vote_settings_enum RENAME TO federation_mode_enum;

