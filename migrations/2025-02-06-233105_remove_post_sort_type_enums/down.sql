-- This removes all the extra post_sort_type_enums,
-- and adds a default_post_time_range_seconds field.
-- Drop the defaults because of a postgres bug
ALTER TABLE local_user
    ALTER default_post_sort_type DROP DEFAULT;

ALTER TABLE local_site
    ALTER default_post_sort_type DROP DEFAULT;

-- Change all the top variants to top in the two tables that use the enum
UPDATE
    local_user
SET
    default_post_sort_type = 'Active'
WHERE
    default_post_sort_type = 'Top';

UPDATE
    local_site
SET
    default_post_sort_type = 'Active'
WHERE
    default_post_sort_type = 'Top';

-- rename the old enum to a tmp name
ALTER TYPE post_sort_type_enum RENAME TO post_sort_type_enum__;

-- create the new enum
CREATE TYPE post_sort_type_enum AS ENUM (
    'Active',
    'Hot',
    'New',
    'Old',
    'TopDay',
    'TopWeek',
    'TopMonth',
    'TopYear',
    'TopAll',
    'MostComments',
    'NewComments',
    'TopHour',
    'TopSixHour',
    'TopTwelveHour',
    'TopThreeMonths',
    'TopSixMonths',
    'TopNineMonths',
    'Controversial',
    'Scaled'
);

-- alter all you enum columns
ALTER TABLE local_user
    ALTER COLUMN default_post_sort_type TYPE post_sort_type_enum
    USING default_post_sort_type::text::post_sort_type_enum;

ALTER TABLE local_site
    ALTER COLUMN default_post_sort_type TYPE post_sort_type_enum
    USING default_post_sort_type::text::post_sort_type_enum;

-- drop the old enum
DROP TYPE post_sort_type_enum__;

-- Add back in the default
ALTER TABLE local_user
    ALTER default_post_sort_type SET DEFAULT 'Active';

ALTER TABLE local_site
    ALTER default_post_sort_type SET DEFAULT 'Active';

-- Drop the new columns
ALTER TABLE local_user
    DROP COLUMN default_post_time_range_seconds;

ALTER TABLE local_site
    DROP COLUMN default_post_time_range_seconds;

