ALTER TABLE community
    ADD COLUMN hidden bool NOT NULL;

UPDATE
    community
SET
    (hidden = TRUE,
        visibility = 'Public')
WHERE
    visibility = 'Hidden';

ALTER TYPE community_visibility RENAME TO community_visibility__;

CREATE TYPE community_visibility AS enum (
    'Public',
    'LocalOnly',
    'Private'
);

ALTER TABLE community
    ALTER COLUMN visibility DROP DEFAULT;

ALTER TABLE community
    ALTER COLUMN visibility TYPE community_visibility
    USING visibility::text::community_visibility;

ALTER TABLE community
    ALTER COLUMN visibility SET DEFAULT 'Public';

DROP TYPE community_visibility__;

