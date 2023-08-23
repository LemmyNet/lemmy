DROP FUNCTION scaled_rank;

ALTER TABLE post_aggregates
    DROP COLUMN scaled_rank;

ALTER TABLE local_user
    ALTER default_sort_type DROP DEFAULT;

-- Remove the 'Scaled' sort enum
UPDATE
    local_user
SET
    default_sort_type = 'Hot'
WHERE
    default_sort_type = 'Scaled';

-- rename the old enum
ALTER TYPE sort_type_enum RENAME TO sort_type_enum__;

-- create the new enum
CREATE TYPE sort_type_enum AS ENUM (
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
    'TopNineMonths'
);

-- alter all your enum columns
ALTER TABLE local_user
    ALTER COLUMN default_sort_type TYPE sort_type_enum
    USING default_sort_type::text::sort_type_enum;

ALTER TABLE local_user
    ALTER default_sort_type SET DEFAULT 'Active';

-- drop the old enum
DROP TYPE sort_type_enum__;

