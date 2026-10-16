ALTER TABLE community
    DROP COLUMN post_upvote_mode,
    DROP COLUMN post_downvote_mode,
    DROP COLUMN comment_upvote_mode,
    DROP COLUMN comment_downvote_mode;

ALTER TABLE local_site RENAME COLUMN default_post_upvotes TO post_upvotes;

ALTER TABLE local_site RENAME COLUMN default_post_downvotes TO post_downvotes;

ALTER TABLE local_site RENAME COLUMN default_comment_upvotes TO comment_upvotes;

ALTER TABLE local_site RENAME COLUMN default_comment_downvotes TO comment_downvotes;

CREATE TYPE federation_mode_enum AS ENUM (
    'All',
    'Local',
    'Disable'
);

ALTER TABLE local_site
    ALTER COLUMN default_post_upvotes TYPE federation_mode_enum
    USING default_post_upvotes::text::federation_mode_enum,
    ALTER COLUMN default_post_downvotes TYPE federation_mode_enum
    USING default_post_downvotes::text::federation_mode_enum,
    ALTER COLUMN default_comment_upvotes TYPE federation_mode_enum
    USING default_comment_upvotes::text::federation_mode_enum,
    ALTER COLUMN default_comment_downvotes TYPE federation_mode_enum
    USING default_comment_downvotes::text::federation_mode_enum;

DROP TYPE vote_settings_enum;

