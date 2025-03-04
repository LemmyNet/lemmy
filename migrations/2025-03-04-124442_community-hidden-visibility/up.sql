ALTER TYPE community_visibility RENAME TO community_visibility__;

CREATE TYPE community_visibility AS enum (
    'Public',
    'LocalOnly',
    'Private',
    'Hidden'
);

ALTER TABLE community
    ALTER COLUMN visibility DROP DEFAULT;

ALTER TABLE community
    ALTER COLUMN visibility TYPE community_visibility
    USING visibility::text::community_visibility;

ALTER TABLE community
    ALTER COLUMN visibility SET DEFAULT 'Public';

DROP TYPE community_visibility__;

UPDATE
    community
SET
    visibility = 'Hidden'
WHERE
    hidden;

ALTER TABLE community
    DROP COLUMN hidden;

