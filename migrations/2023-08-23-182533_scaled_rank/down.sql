DROP FUNCTION scaled_rank;

ALTER TABLE community_aggregates
    ALTER COLUMN hot_rank TYPE integer,
    ALTER COLUMN hot_rank SET DEFAULT 1728;

ALTER TABLE comment_aggregates
    ALTER COLUMN hot_rank TYPE integer,
    ALTER COLUMN hot_rank SET DEFAULT 1728;

ALTER TABLE post_aggregates
    ALTER COLUMN hot_rank TYPE integer,
    ALTER COLUMN hot_rank SET DEFAULT 1728,
    ALTER COLUMN hot_rank_active TYPE integer,
    ALTER COLUMN hot_rank_active SET DEFAULT 1728;

-- Change back to integer version
DROP FUNCTION hot_rank (numeric, published timestamp with time zone);

CREATE OR REPLACE FUNCTION hot_rank (score numeric, published timestamp with time zone)
    RETURNS integer
    AS $$
DECLARE
    hours_diff numeric := EXTRACT(EPOCH FROM (now() - published)) / 3600;
BEGIN
    IF (hours_diff > 0) THEN
        RETURN floor(10000 * log(greatest (1, score + 3)) / power((hours_diff + 2), 1.8))::integer;
    ELSE
        -- if the post is from the future, set hot score to 0. otherwise you can game the post to
        -- always be on top even with only 1 vote by setting it to the future
        RETURN 0;
    END IF;
END;
$$
LANGUAGE plpgsql
IMMUTABLE PARALLEL SAFE;

ALTER TABLE post_aggregates
    DROP COLUMN scaled_rank;

-- The following code is necessary because postgres can't remove
-- a single enum value.
ALTER TABLE local_user
    ALTER default_sort_type DROP DEFAULT;

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

