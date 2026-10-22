ALTER TYPE federation_mode_enum RENAME TO vote_settings_enum;

ALTER TYPE vote_settings_enum
    ADD VALUE 'Subscribed';

ALTER TABLE local_site RENAME COLUMN post_upvotes TO default_post_upvotes;

ALTER TABLE local_site RENAME COLUMN post_downvotes TO default_post_downvotes;

ALTER TABLE local_site RENAME COLUMN comment_upvotes TO default_comment_upvotes;

ALTER TABLE local_site RENAME COLUMN comment_downvotes TO default_comment_downvotes;

ALTER TABLE community
    ADD COLUMN post_upvotes vote_settings_enum DEFAULT 'All'::vote_settings_enum NOT NULL,
    ADD COLUMN post_downvotes vote_settings_enum DEFAULT 'All'::vote_settings_enum NOT NULL,
    ADD COLUMN comment_upvotes vote_settings_enum DEFAULT 'All'::vote_settings_enum NOT NULL,
    ADD COLUMN comment_downvotes vote_settings_enum DEFAULT 'All'::vote_settings_enum NOT NULL;

