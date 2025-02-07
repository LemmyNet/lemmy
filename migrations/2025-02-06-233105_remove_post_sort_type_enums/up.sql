-- This removes all the extra post_sort_type_enums,
-- and adds a default_post_time_range_seconds field.

-- Add a Top value to the existing enum
ALTER TYPE post_sort_type_enum ADD VALUE 'Top';

-- Change all the top variants to top in the two tables that use the enum
UPDATE
    local_user
SET
    default_post_sort_type = 'Top'
WHERE
    default_post_sort_type IN ('TopDay','TopWeek', 'TopMonth', 'TopYear', 'TopAll', 'TopHour', 'TopSixHour', 'TopTwelveHour', 'TopThreeMonths', 'TopSixMonths', 'TopNineMonths');

UPDATE
    local_site
SET
    default_post_sort_type = 'Top'
WHERE
    default_post_sort_type IN ('TopDay','TopWeek', 'TopMonth', 'TopYear', 'TopAll', 'TopHour', 'TopSixHour', 'TopTwelveHour', 'TopThreeMonths', 'TopSixMonths', 'TopNineMonths');

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

-- drop the old enum
DROP TYPE post_sort_type_enum__;

-- Add the new column to both tables (null means no limit)
alter table local_user
    ADD COLUMN default_post_time_range_seconds INTEGER;

alter table local_site
    ADD COLUMN default_post_time_range_seconds INTEGER;


