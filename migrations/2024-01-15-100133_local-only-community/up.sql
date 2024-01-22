CREATE TYPE community_visibility AS enum (
    'public',
    'local-only'
);

ALTER TABLE community
    ADD COLUMN visibility community_visibility NOT NULL DEFAULT 'public';

