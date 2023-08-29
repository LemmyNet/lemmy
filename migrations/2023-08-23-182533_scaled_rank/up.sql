-- Change hot ranks and functions from an int to a float
ALTER TABLE community_aggregates
    ALTER COLUMN hot_rank TYPE float,
    ALTER COLUMN hot_rank SET DEFAULT 0.1728;

ALTER TABLE comment_aggregates
    ALTER COLUMN hot_rank TYPE float,
    ALTER COLUMN hot_rank SET DEFAULT 0.1728;

ALTER TABLE post_aggregates
    ALTER COLUMN hot_rank TYPE float,
    ALTER COLUMN hot_rank SET DEFAULT 0.1728,
    ALTER COLUMN hot_rank_active TYPE float,
    ALTER COLUMN hot_rank_active SET DEFAULT 0.1728;

DROP FUNCTION hot_rank (numeric, published timestamp with time zone);

CREATE OR REPLACE FUNCTION hot_rank (score numeric, published timestamp with time zone)
    RETURNS float
    AS $$
DECLARE
    hours_diff numeric := EXTRACT(EPOCH FROM (now() - published)) / 3600;
BEGIN
    IF (hours_diff > 0) THEN
        RETURN log(greatest (1, score + 3)) / power((hours_diff + 2), 1.8);
    ELSE
        -- if the post is from the future, set hot score to 0. otherwise you can game the post to
        -- always be on top even with only 1 vote by setting it to the future
        RETURN 0.0;
    END IF;
END;
$$
LANGUAGE plpgsql
IMMUTABLE PARALLEL SAFE;

-- The new scaled rank function
CREATE OR REPLACE FUNCTION scaled_rank (score numeric, published timestamp with time zone, users_active_month numeric)
    RETURNS float
    AS $$
BEGIN
    -- Add 2 to avoid divide by zero errors
    -- Default for score = 1, active users = 1, and now, is (0.1728 / log(2 + 1)) = 0.3621
    RETURN (hot_rank (score, published) / log(2 + users_active_month));
END;
$$
LANGUAGE plpgsql
IMMUTABLE PARALLEL SAFE;

-- TODO figure out correct default
ALTER TABLE post_aggregates
    ADD COLUMN scaled_rank float NOT NULL DEFAULT 0.3621;

CREATE INDEX idx_post_aggregates_featured_community_scaled ON post_aggregates (featured_community DESC, scaled_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_local_scaled ON post_aggregates (featured_local DESC, scaled_rank DESC, published DESC);

-- We forgot to add the controversial sort type
ALTER TYPE sort_type_enum
    ADD VALUE 'Controversial';

-- Add the Scaled enum
ALTER TYPE sort_type_enum
    ADD VALUE 'Scaled';

