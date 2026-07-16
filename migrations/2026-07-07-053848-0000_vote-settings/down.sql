ALTER TABLE community
    DROP COLUMN downvote_mode;

CREATE TYPE federation_mode_enum AS ENUM (
    'All',
    'Local',
    'Disable'
);

ALTER TABLE local_site
    ALTER post_upvotes DROP DEFAULT,
    ALTER post_downvotes DROP DEFAULT,
    ALTER comment_upvotes DROP DEFAULT,
    ALTER comment_downvotes DROP DEFAULT;

ALTER TABLE local_site
    ALTER COLUMN post_upvotes TYPE federation_mode_enum
    USING post_upvotes::text::federation_mode_enum,
    ALTER COLUMN post_downvotes TYPE federation_mode_enum
    USING post_downvotes::text::federation_mode_enum,
    ALTER COLUMN comment_upvotes TYPE federation_mode_enum
    USING comment_upvotes::text::federation_mode_enum,
    ALTER COLUMN comment_downvotes TYPE federation_mode_enum
    USING comment_downvotes::text::federation_mode_enum;

ALTER TABLE local_site
    ALTER post_upvotes SET DEFAULT 'All',
    ALTER post_downvotes SET DEFAULT 'All',
    ALTER comment_upvotes SET DEFAULT 'All',
    ALTER comment_downvotes SET DEFAULT 'All';

DROP TYPE vote_settings_enum;

