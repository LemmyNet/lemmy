-- recreate columns in the original order
ALTER TABLE community
    ADD COLUMN hidden bool DEFAULT FALSE NOT NULL,
    ADD COLUMN visibility_new community_visibility NOT NULL DEFAULT 'Public';

UPDATE
    community
SET
    visibility_new = visibility;

ALTER TABLE community
    DROP COLUMN visibility;

ALTER TABLE community RENAME COLUMN visibility_new TO visibility;

-- same changes as up.sql, but the other way round
UPDATE
    community
SET
    (hidden,
        visibility) = (TRUE,
        'Public')
WHERE
    visibility = 'Unlisted';

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

REINDEX TABLE community;

-- revert modlog table changes
CREATE TABLE mod_hide_community (
    id serial PRIMARY KEY,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    mod_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    reason text,
    hidden boolean DEFAULT FALSE NOT NULL
);

ALTER TABLE modlog_combined
    DROP COLUMN mod_change_community_visibility_id,
    ADD COLUMN mod_hide_community_id int REFERENCES mod_hide_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_lock_post_id_new int,
    ADD COLUMN mod_remove_comment_id_new int,
    ADD COLUMN mod_remove_community_id_new int,
    ADD COLUMN mod_remove_post_id_new int,
    ADD COLUMN mod_transfer_community_id_new int;

UPDATE
    modlog_combined
SET
    (mod_lock_post_id_new,
        mod_remove_comment_id_new,
        mod_remove_community_id_new,
        mod_remove_post_id_new,
        mod_transfer_community_id_new) = (mod_lock_post_id,
        mod_remove_comment_id,
        mod_remove_community_id,
        mod_remove_post_id,
        mod_transfer_community_id);

ALTER TABLE modlog_combined
    DROP COLUMN mod_lock_post_id,
    DROP COLUMN mod_remove_comment_id,
    DROP COLUMN mod_remove_community_id,
    DROP COLUMN mod_remove_post_id,
    DROP COLUMN mod_transfer_community_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_lock_post_id_new TO mod_lock_post_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_remove_comment_id_new TO mod_remove_comment_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_remove_community_id_new TO mod_remove_community_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_remove_post_id_new TO mod_remove_post_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_transfer_community_id_new TO mod_transfer_community_id;

ALTER TABLE modlog_combined
    ADD CONSTRAINT modlog_combined_mod_hide_community_id_key UNIQUE (mod_hide_community_id),
    ADD CONSTRAINT modlog_combined_mod_lock_post_id_key UNIQUE (mod_lock_post_id),
    ADD CONSTRAINT modlog_combined_mod_remove_comment_id_key UNIQUE (mod_remove_comment_id),
    ADD CONSTRAINT modlog_combined_mod_remove_community_id_key UNIQUE (mod_remove_community_id),
    ADD CONSTRAINT modlog_combined_mod_remove_post_id_key UNIQUE (mod_remove_post_id),
    ADD CONSTRAINT modlog_combined_mod_transfer_community_id_key UNIQUE (mod_transfer_community_id),
    ADD CONSTRAINT modlog_combined_mod_lock_post_id_fkey FOREIGN KEY (mod_lock_post_id) REFERENCES mod_lock_post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_mod_remove_comment_id_fkey FOREIGN KEY (mod_remove_comment_id) REFERENCES mod_remove_comment (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_mod_remove_community_id_fkey FOREIGN KEY (mod_remove_community_id) REFERENCES mod_remove_community (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_mod_remove_post_id_fkey FOREIGN KEY (mod_remove_post_id) REFERENCES mod_remove_post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_mod_transfer_community_id_fkey FOREIGN KEY (mod_transfer_community_id) REFERENCES mod_transfer_community (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_check CHECK ((num_nonnulls (admin_allow_instance_id, admin_block_instance_id, admin_purge_comment_id, admin_purge_community_id, admin_purge_person_id, admin_purge_post_id, mod_add_id, mod_add_community_id, mod_ban_id, mod_ban_from_community_id, mod_feature_post_id, mod_hide_community_id, mod_lock_post_id, mod_remove_comment_id, mod_remove_community_id, mod_remove_post_id, mod_transfer_community_id) = 1));

DROP TABLE mod_change_community_visibility;

DROP TYPE community_visibility__;

