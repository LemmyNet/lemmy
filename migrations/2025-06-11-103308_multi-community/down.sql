ALTER TABLE search_combined
    DROP CONSTRAINT search_combined_check;

ALTER TABLE search_combined
    ADD CONSTRAINT search_combined_check CHECK (num_nonnulls (post_id, comment_id, community_id, person_id) = 1);

ALTER TABLE search_combined
    DROP COLUMN multi_community_id;

ALTER TABLE local_site
    DROP COLUMN suggested_communities;

DROP TABLE multi_community_follow;

DROP TABLE multi_community_entry;

DROP TABLE multi_community;

CREATE TYPE listing_type_enum_tmp AS ENUM (
    'All',
    'Local',
    'Subscribed',
    'ModeratorView'
);

UPDATE
    local_user
SET
    default_listing_type = 'All'
WHERE
    default_listing_type = 'Suggested';

UPDATE
    local_site
SET
    default_post_listing_type = 'All'
WHERE
    default_post_listing_type = 'Suggested';

ALTER TABLE local_user
    ALTER COLUMN default_listing_type DROP DEFAULT,
    ALTER COLUMN default_listing_type TYPE listing_type_enum_tmp
    USING (default_listing_type::text::listing_type_enum_tmp),
    ALTER COLUMN default_listing_type SET DEFAULT 'Local';

ALTER TABLE local_site
    ALTER COLUMN default_post_listing_type DROP DEFAULT,
    ALTER COLUMN default_post_listing_type TYPE listing_type_enum_tmp
    USING (default_post_listing_type::text::listing_type_enum_tmp),
    ALTER COLUMN default_post_listing_type SET DEFAULT 'Local',
    DROP COLUMN multi_comm_follower;

DROP TYPE listing_type_enum;

ALTER TYPE listing_type_enum_tmp RENAME TO listing_type_enum;

