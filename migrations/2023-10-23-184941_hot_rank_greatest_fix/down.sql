CREATE OR REPLACE FUNCTION hot_rank (score numeric, published timestamp with time zone)
    RETURNS float
    AS $$
DECLARE
    hours_diff numeric := EXTRACT(EPOCH FROM (now() - published)) / 3600;
BEGIN
    -- 24 * 7 = 168, so after a week, it will default to 0.
    IF (hours_diff > 0 AND hours_diff < 168) THEN
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
