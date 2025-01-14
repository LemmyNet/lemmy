ALTER TYPE community_visibility
    ADD value 'Private';

-- Change `community_follower.pending` to `state` enum
CREATE TYPE community_follower_state AS enum (
    'Accepted',
    'Pending',
    'ApprovalRequired'
);

ALTER TABLE community_follower
    ALTER COLUMN pending DROP DEFAULT;

CREATE OR REPLACE FUNCTION convert_follower_state (b bool)
    RETURNS community_follower_state
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE
    AS $$
    SELECT
        CASE WHEN b = TRUE THEN
            'Pending'::community_follower_state
        ELSE
            'Accepted'::community_follower_state
        END
$$;

ALTER TABLE community_follower
    ALTER COLUMN pending TYPE community_follower_state
    USING convert_follower_state (pending);

DROP FUNCTION convert_follower_state;

ALTER TABLE community_follower RENAME COLUMN pending TO state;

-- Add column for mod who approved the private community follower
-- Dont use foreign key here, otherwise joining to person table doesnt work easily
ALTER TABLE community_follower
    ADD COLUMN approver_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE;

-- Enable signed fetch, necessary to fetch content in private communities
ALTER TABLE ONLY local_site
    ALTER COLUMN federation_signed_fetch SET DEFAULT TRUE;

UPDATE
    local_site
SET
    federation_signed_fetch = TRUE;

