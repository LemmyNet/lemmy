-- The hot_rank algorithm currently uses greatest(1, score + 3)
-- This greatest of 1 incorrect because log10(1) is zero,
-- so it will push negative-voted comments / posts to the bottom, IE hot_rank = 0
-- The update_scheduled_ranks will never recalculate them, because it ignores content
-- with hot_rank = 0
CREATE OR REPLACE FUNCTION hot_rank (score numeric, published timestamp with time zone)
    RETURNS float
    AS $$
DECLARE
    hours_diff numeric := EXTRACT(EPOCH FROM (now() - published)) / 3600;
BEGIN
    -- 24 * 7 = 168, so after a week, it will default to 0.
    IF (hours_diff > 0 AND hours_diff < 168) THEN
        -- Use greatest(2,score), so that the hot_rank will be positive and not ignored.
        RETURN log(greatest (2, score)) / power((hours_diff + 2), 1.8);
    ELSE
        -- if the post is from the future, set hot score to 0. otherwise you can game the post to
        -- always be on top even with only 1 vote by setting it to the future
        RETURN 0.0;
    END IF;
END;
$$
LANGUAGE plpgsql
IMMUTABLE PARALLEL SAFE;

