CREATE TYPE community_visibility AS enum (
    'Public',
    'LocalOnly'
);

ALTER TABLE community
    ADD COLUMN visibility community_visibility NOT NULL DEFAULT 'Public';

