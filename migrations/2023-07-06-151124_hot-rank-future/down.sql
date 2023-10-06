CREATE OR REPLACE FUNCTION hot_rank (score numeric, published timestamp without time zone)
    RETURNS integer
    AS $$
BEGIN
    -- hours_diff:=EXTRACT(EPOCH FROM (timezone('utc',now()) - published))/3600
    RETURN floor(10000 * log(greatest (1, score + 3)) / power(((EXTRACT(EPOCH FROM (timezone('utc', now()) - published)) / 3600) + 2), 1.8))::integer;
END;
$$
LANGUAGE plpgsql
IMMUTABLE;

