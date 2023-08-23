CREATE OR REPLACE FUNCTION scaled_rank (score numeric, published timestamp without time zone, users_active_month numeric)
    RETURNS integer
    AS $$
BEGIN
    -- Add 2 to avoid divide by zero errors
    -- Use 0.1 to lessen the initial sharp decline at a hot_rank ~ 300
    -- Default for score = 1, active users = 1, and now, is 742
    RETURN (hot_rank (score, published) / log(2 + 0.1 * users_active_month))::integer;
END;
$$
LANGUAGE plpgsql
IMMUTABLE PARALLEL SAFE;

ALTER TABLE post_aggregates
    ADD COLUMN scaled_rank integer NOT NULL DEFAULT 742;

CREATE INDEX idx_post_aggregates_featured_community_scaled ON post_aggregates (featured_community DESC, scaled_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_local_scaled ON post_aggregates (featured_local DESC, scaled_rank DESC, published DESC);

-- We forgot to add the controversial sort type
ALTER TYPE sort_type_enum
    ADD VALUE 'Controversial';

-- Add the Scaled enum
ALTER TYPE sort_type_enum
    ADD VALUE 'Scaled';

