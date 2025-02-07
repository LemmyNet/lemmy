-- This removes all the extra post_sort_type_enums,
-- and adds a default_post_time_range_seconds field.
-- Change all the top variants to top in the two tables that use the enum
-- Because of a postgres bug, you can't assign this to a new enum value,
-- unless you run an unsafe commit first. So just use active.
-- https://dba.stackexchange.com/questions/280371/postgres-unsafe-use-of-new-value-of-enum-type
UPDATE
    local_user
SET
    default_post_sort_type = 'Active'
WHERE
    default_post_sort_type IN ('TopDay', 'TopWeek', 'TopMonth', 'TopYear', 'TopAll', 'TopHour', 'TopSixHour', 'TopTwelveHour', 'TopThreeMonths', 'TopSixMonths', 'TopNineMonths');

UPDATE
    local_site
SET
    default_post_sort_type = 'Active'
WHERE
    default_post_sort_type IN ('TopDay', 'TopWeek', 'TopMonth', 'TopYear', 'TopAll', 'TopHour', 'TopSixHour', 'TopTwelveHour', 'TopThreeMonths', 'TopSixMonths', 'TopNineMonths');

-- Drop the defaults because of a postgres bug
ALTER TABLE local_user
    ALTER default_post_sort_type DROP DEFAULT;

ALTER TABLE local_site
    ALTER default_post_sort_type DROP DEFAULT;

-- rename the old enum to a tmp name
ALTER TYPE post_sort_type_enum RENAME TO post_sort_type_enum__;

-- create the new enum
CREATE TYPE post_sort_type_enum AS ENUM (
    'Active',
    'Hot',
    'New',
    'Old',
    'Top',
    'MostComments',
    'NewComments',
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

-- Add the new column to both tables (null means no limit)
ALTER TABLE local_user
    ADD COLUMN default_post_time_range_seconds INTEGER;

ALTER TABLE local_site
    ADD COLUMN default_post_time_range_seconds INTEGER;

