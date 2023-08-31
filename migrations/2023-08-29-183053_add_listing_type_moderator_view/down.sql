ALTER TABLE local_user
    ALTER default_listing_type DROP DEFAULT;

ALTER TABLE local_site
    ALTER default_post_listing_type DROP DEFAULT;

UPDATE
    local_user
SET
    default_listing_type = 'Local'
WHERE
    default_listing_type = 'ModeratorView';

UPDATE
    local_site
SET
    default_post_listing_type = 'Local'
WHERE
    default_post_listing_type = 'ModeratorView';

-- rename the old enum
ALTER TYPE listing_type_enum RENAME TO listing_type_enum__;

-- create the new enum
CREATE TYPE listing_type_enum AS ENUM (
    'All',
    'Local',
    'Subscribed'
);

-- alter all your enum columns
ALTER TABLE local_user
    ALTER COLUMN default_listing_type TYPE listing_type_enum
    USING default_listing_type::text::listing_type_enum;

ALTER TABLE local_site
    ALTER COLUMN default_post_listing_type TYPE listing_type_enum
    USING default_post_listing_type::text::listing_type_enum;

-- Add back in the default
ALTER TABLE local_user
    ALTER default_listing_type SET DEFAULT 'Local';

ALTER TABLE local_site
    ALTER default_post_listing_type SET DEFAULT 'Local';

-- drop the old enum
DROP TYPE listing_type_enum__;

