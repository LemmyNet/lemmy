CREATE OR REPLACE FUNCTION hot_rank(score numeric, published timestamp without time zone)
    RETURNS integer
    AS $$
DECLARE
    hours_diff numeric := EXTRACT(EPOCH FROM (timezone('utc', now()) - published)) / 3600;
BEGIN
    IF (hours_diff > 0) THEN
        RETURN floor(10000 * log(greatest(1, score + 3)) / power((hours_diff + 2), 1.8))::integer;
    ELSE
        RETURN 0;
    END IF;
END;
$$
LANGUAGE plpgsql
IMMUTABLE PARALLEL SAFE;

