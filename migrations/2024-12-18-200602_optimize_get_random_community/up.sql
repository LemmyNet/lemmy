-- * inclusive bounds of `smallint` range from https://www.postgresql.org/docs/17/datatype-numeric.html
-- * built-in `random` function has `VOLATILE` and `PARALLEL RESTRICTED` according to:
--   * https://www.postgresql.org/docs/current/parallel-safety.html#PARALLEL-LABELING
--   * https://www.postgresql.org/docs/17/xfunc-volatility.html
CREATE FUNCTION random_smallint ()
    RETURNS smallint
    LANGUAGE sql
    VOLATILE PARALLEL RESTRICTED RETURN
    -- https://stackoverflow.com/questions/1400505/generate-a-random-number-in-the-range-1-10/1400752#1400752
    -- (65536 = exclusive upper bound - inclusive lower bound)
    trunc ((random() * (65536)) - 32768
);

ALTER TABLE community
    ADD COLUMN random_number smallint NOT NULL DEFAULT random_smallint ();

CREATE INDEX idx_community_random_number ON community (random_number) INCLUDE (local, nsfw)
WHERE
    NOT (deleted OR removed OR visibility = 'Private');

