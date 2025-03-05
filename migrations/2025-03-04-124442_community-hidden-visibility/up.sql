--
-- create enum with new values and drop old one
CREATE TYPE community_visibility AS enum (
    'Public',
    'LocalOnly',
    'Private',
    'Hidden'
);

ALTER TYPE community_visibility RENAME TO community_visibility__;

-- drop default value and index which reference old enum
ALTER TABLE community
    ALTER COLUMN visibility DROP DEFAULT;

DROP INDEX idx_community_random_number;

-- change the column type
ALTER TABLE community
    ALTER COLUMN visibility TYPE community_visibility
    USING visibility::text::community_visibility;

-- add default and index back in
ALTER TABLE community
    ALTER COLUMN visibility SET DEFAULT 'Public';

CREATE INDEX idx_community_random_number ON community (random_number) INCLUDE (local, nsfw)
WHERE
    NOT (deleted OR removed OR visibility = 'Private' OR visibility = 'Hidden');

DROP TYPE community_visibility__;

-- write hidden value to visibility column
UPDATE
    community
SET
    visibility = 'Hidden'
WHERE
    hidden;

-- drop the old hidden column
ALTER TABLE community
    DROP COLUMN hidden;

