--  Add the column back
ALTER TABLE community
    ADD COLUMN creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE;

-- Recreate the index
CREATE INDEX idx_community_creator ON community (creator_id);

-- Add the data, selecting the highest mod
UPDATE
    community
SET
    creator_id = sub.person_id
FROM (
    SELECT
        cm.community_id,
        cm.person_id
    FROM
        community_moderator cm
    LIMIT 1) AS sub
WHERE
    id = sub.community_id;

-- Set to not null
ALTER TABLE community
    ALTER COLUMN creator_id SET NOT NULL;

