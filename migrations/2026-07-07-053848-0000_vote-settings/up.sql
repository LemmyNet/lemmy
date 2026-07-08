ALTER TABLE local_site
    DROP COLUMN post_upvotes;

ALTER TABLE local_site
    DROP COLUMN post_downvotes;

ALTER TABLE local_site
    DROP COLUMN comment_upvotes;

ALTER TABLE local_site
    DROP COLUMN comment_downvotes;

DROP TYPE federation_mode_enum;

-- TODO: naming of variants is a bit unclear/inconsistent
CREATE TYPE community_downvote_enum AS ENUM (
    'All',
    'Subscribed',
    'Disabled'
);

ALTER TABLE community
    ADD COLUMN downvote_mode community_downvote_enum DEFAULT 'All'::community_downvote_enum NOT NULL;

