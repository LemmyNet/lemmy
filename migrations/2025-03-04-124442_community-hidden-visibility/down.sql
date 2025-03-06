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
    ADD COLUMN mod_hide_community_id int REFERENCES mod_hide_community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
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

ALTER TABLE modlog_combined
    ALTER COLUMN mod_lock_post_id SET NOT NULL;

ALTER TABLE modlog_combined
    ALTER COLUMN mod_remove_comment_id SET NOT NULL;

ALTER TABLE modlog_combined
    ALTER COLUMN mod_remove_community_id SET NOT NULL;

ALTER TABLE modlog_combined
    ALTER COLUMN mod_remove_post_id SET NOT NULL;

ALTER TABLE modlog_combined
    ALTER COLUMN mod_transfer_community_id SET NOT NULL;

DROP TABLE mod_change_community_visibility;

DROP TYPE community_visibility__;

