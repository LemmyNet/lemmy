-- recreate columns in the original order
ALTER TABLE community
    ADD COLUMN hidden bool DEFAULT FALSE NOT NULL,
    ADD COLUMN posting_restricted_to_mods_new bool NOT NULL DEFAULT FALSE,
    ADD COLUMN instance_id_new int NOT NULL,
    ADD COLUMN moderators_url_new varchar(255),
    ADD COLUMN featured_url_new varchar(255),
    ADD COLUMN visibility_new community_visibility NOT NULL DEFAULT 'Public',
    ADD COLUMN description_new varchar(150),
    ADD COLUMN random_number_new smallint NOT NULL DEFAULT random_smallint ();

UPDATE
    community
SET
    (posting_restricted_to_mods_new,
        instance_id_new,
        moderators_url_new,
        featured_url_new,
        visibility_new,
        description_new,
        random_number_new) = (posting_restricted_to_mods,
        instance_id,
        moderators_url,
        featured_url,
        visibility,
        description,
        random_number);

ALTER TABLE community
    DROP COLUMN posting_restricted_to_mods,
    DROP COLUMN instance_id,
    DROP COLUMN moderators_url,
    DROP COLUMN featured_url,
    DROP COLUMN visibility,
    DROP COLUMN description,
    DROP COLUMN random_number;

ALTER TABLE community RENAME COLUMN posting_restricted_to_mods_new TO posting_restricted_to_mods;

ALTER TABLE community RENAME COLUMN instance_id_new TO instance_id;

ALTER TABLE community RENAME COLUMN moderators_url_new TO moderators_url;

ALTER TABLE community RENAME COLUMN featured_url_new TO featured_url;

ALTER TABLE community RENAME COLUMN visibility_new TO visibility;

ALTER TABLE community RENAME COLUMN description_new TO description;

ALTER TABLE community RENAME COLUMN random_number_new TO random_number;

ALTER TABLE community
    ADD CONSTRAINT community_featured_url_key UNIQUE (featured_url),
    ADD CONSTRAINT community_moderators_url_key UNIQUE (moderators_url),
    ADD CONSTRAINT community_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE;

REINDEX TABLE community;

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

ALTER TABLE community
    ALTER COLUMN visibility TYPE community_visibility
    USING visibility::text::community_visibility;

ALTER TABLE community
    ALTER COLUMN visibility SET DEFAULT 'Public';

CREATE INDEX idx_community_random_number ON community (random_number) INCLUDE (local, nsfw)
WHERE
    NOT (deleted OR removed OR visibility = 'Private');

DROP TYPE community_visibility__;

