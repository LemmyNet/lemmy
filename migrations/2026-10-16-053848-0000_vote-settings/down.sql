ALTER TABLE community
    DROP COLUMN post_upvotes,
    DROP COLUMN post_downvotes,
    DROP COLUMN comment_upvotes,
    DROP COLUMN comment_downvotes;

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
    ALTER COLUMN post_upvotes TYPE federation_mode_enum
    USING post_upvotes::text::federation_mode_enum,
    ALTER COLUMN post_downvotes TYPE federation_mode_enum
    USING post_downvotes::text::federation_mode_enum,
    ALTER COLUMN comment_upvotes TYPE federation_mode_enum
    USING comment_upvotes::text::federation_mode_enum,
    ALTER COLUMN comment_downvotes TYPE federation_mode_enum
    USING comment_downvotes::text::federation_mode_enum;

DROP TYPE vote_settings_enum;

