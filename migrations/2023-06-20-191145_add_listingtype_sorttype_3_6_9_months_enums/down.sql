ALTER TABLE local_user
    ALTER default_sort_type DROP DEFAULT;

-- update the default sort type
UPDATE
    local_user
SET
    default_sort_type = 'TopDay'
WHERE
    default_sort_type IN ('TopThreeMonths', 'TopSixMonths', 'TopNineMonths');

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
    'TopTwelveHour'
);

-- alter all you enum columns
ALTER TABLE local_user
    ALTER COLUMN default_sort_type TYPE sort_type_enum
    USING default_sort_type::text::sort_type_enum;

ALTER TABLE local_user
    ALTER default_sort_type SET DEFAULT 'Active';

-- drop the old enum
DROP TYPE sort_type_enum__;

