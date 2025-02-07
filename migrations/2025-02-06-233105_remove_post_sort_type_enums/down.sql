-- This removes all the extra post_sort_type_enums,
-- and adds a default_post_time_range_seconds field.

-- Add a temp TopAll value back
ALTER TYPE post_sort_type_enum ADD VALUE 'TopAll';


-- Change all the top variants to top in the two tables that use the enum
UPDATE
    local_user
SET
    default_post_sort_type = 'TopAll'
WHERE
    default_post_sort_type = 'Top';

UPDATE
    local_site
SET
    default_post_sort_type = 'TopAll'
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

-- drop the old enum
DROP TYPE post_sort_type_enum__;

-- Drop the new columns
alter table local_user
    DROP COLUMN default_post_time_range_seconds;

alter table local_site
    DROP COLUMN default_post_time_range_seconds;


