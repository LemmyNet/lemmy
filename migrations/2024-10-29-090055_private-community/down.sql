-- Remove private visibility
ALTER TYPE community_visibility RENAME TO community_visibility__;

CREATE TYPE community_visibility AS enum (
    'Public',
    'LocalOnly'
);

ALTER TABLE community
    ALTER COLUMN visibility DROP DEFAULT;

ALTER TABLE community
    ALTER COLUMN visibility TYPE community_visibility
    USING visibility::text::community_visibility;

ALTER TABLE community
    ALTER COLUMN visibility SET DEFAULT 'Public';

DROP TYPE community_visibility__;

-- Revert community follower changes
CREATE OR REPLACE FUNCTION convert_follower_state (s community_follower_state)
    RETURNS bool
    LANGUAGE sql
    AS $$
    SELECT
        CASE WHEN s = 'Pending' THEN
            TRUE
        ELSE
            FALSE
        END
$$;

ALTER TABLE community_follower
    ALTER COLUMN state TYPE bool
    USING convert_follower_state (state);

DROP FUNCTION convert_follower_state;

ALTER TABLE community_follower
    ALTER COLUMN state SET DEFAULT FALSE;

ALTER TABLE community_follower RENAME COLUMN state TO pending;

DROP TYPE community_follower_state;

ALTER TABLE community_follower
    DROP COLUMN approver_id;

ALTER TABLE ONLY local_site
    ALTER COLUMN federation_signed_fetch SET DEFAULT FALSE;

