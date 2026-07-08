ALTER TABLE community
    DROP COLUMN downvote_mode;

DROP TYPE community_downvote_enum;

CREATE TYPE federation_mode_enum AS ENUM (
    'All',
    'Local',
    'Disable'
);

-- Add the new columns
ALTER TABLE local_site
    ADD COLUMN post_upvotes federation_mode_enum DEFAULT 'All'::federation_mode_enum NOT NULL,
    ADD COLUMN post_downvotes federation_mode_enum DEFAULT 'All'::federation_mode_enum NOT NULL,
    ADD COLUMN comment_upvotes federation_mode_enum DEFAULT 'All'::federation_mode_enum NOT NULL,
    ADD COLUMN comment_downvotes federation_mode_enum DEFAULT 'All'::federation_mode_enum NOT NULL;

