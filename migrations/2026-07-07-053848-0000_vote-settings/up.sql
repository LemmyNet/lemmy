ALTER TYPE federation_mode_enum RENAME TO vote_settings_enum;

ALTER TYPE vote_settings_enum
    ADD VALUE 'subscribed';

ALTER TABLE community
    ADD COLUMN downvote_mode vote_settings_enum DEFAULT 'All'::vote_settings_enum NOT NULL;

