-- Change community.visibility to allow values:
-- ('Public', 'LocalOnlyPublic', 'LocalOnlyPrivate','Private', 'Hidden')
-- rename old enum and add new one
ALTER TYPE community_visibility RENAME TO community_visibility__;

CREATE TYPE community_visibility AS enum (
    'Public',
    'LocalOnlyPublic',
    'LocalOnly',
    'Private',
    'Unlisted'
);

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
    NOT (deleted OR removed OR visibility = 'Private' OR visibility = 'Unlisted');

DROP TYPE community_visibility__ CASCADE;

ALTER TYPE community_visibility RENAME VALUE 'LocalOnly' TO 'LocalOnlyPrivate';

-- write hidden value to visibility column
UPDATE
    community
SET
    visibility = 'Unlisted'
WHERE
    hidden;

-- drop the old hidden column
ALTER TABLE community
    DROP COLUMN hidden;

-- change modlog tables
ALTER TABLE modlog_combined
    DROP COLUMN mod_hide_community_id;

DROP TABLE mod_hide_community;

CREATE TABLE mod_change_community_visibility (
    id serial PRIMARY KEY,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    mod_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    reason text,
    visibility community_visibility NOT NULL
);

ALTER TABLE modlog_combined
    ADD COLUMN mod_change_community_visibility_id int REFERENCES mod_change_community_visibility (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_check CHECK ((num_nonnulls (admin_allow_instance_id, admin_block_instance_id, admin_purge_comment_id, admin_purge_community_id, admin_purge_person_id, admin_purge_post_id, mod_add_id, mod_add_community_id, mod_ban_id, mod_ban_from_community_id, mod_feature_post_id, mod_change_community_visibility_id, mod_lock_post_id, mod_remove_comment_id, mod_remove_community_id, mod_remove_post_id, mod_transfer_community_id) = 1));

