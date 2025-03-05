ALTER TABLE community
    ADD COLUMN hidden bool DEFAULT FALSE NOT NULL;

-- same changes as up.sql, but the other way round
UPDATE
    community
SET
    (hidden,
        visibility) = (TRUE,
        'Public')
WHERE
    visibility = 'Hidden';

ALTER TYPE community_visibility RENAME VALUE 'LocalOnlyPrivate' TO 'LocalOnly';

ALTER TYPE community_visibility RENAME TO community_visibility__;

CREATE TYPE community_visibility AS enum (
    'Public',
    'LocalOnly',
    'Private'
);

ALTER TABLE community
    ALTER COLUMN visibility DROP DEFAULT;

DROP INDEX idx_community_random_number;

ALTER TABLE community
    ALTER COLUMN visibility TYPE community_visibility
    USING visibility::text::community_visibility;

ALTER TABLE community
    ALTER COLUMN visibility SET DEFAULT 'Public';

CREATE INDEX idx_community_random_number ON community (random_number) INCLUDE (local, nsfw)
WHERE
    NOT (deleted OR removed OR visibility = 'Private');

DROP TYPE community_visibility__;

