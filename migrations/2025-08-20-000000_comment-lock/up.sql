ALTER TABLE comment
    ADD COLUMN "locked" bool NOT NULL DEFAULT FALSE;

CREATE TABLE mod_lock_comment (
    id serial PRIMARY KEY,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id integer NOT NULL REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    locked boolean NOT NULL DEFAULT TRUE,
    reason text,
    published_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_mod_lock_comment_mod ON mod_lock_comment (mod_person_id);

CREATE INDEX idx_mod_lock_comment_comment ON mod_lock_comment (comment_id);

ALTER TABLE modlog_combined
    ADD COLUMN mod_lock_comment_id integer UNIQUE REFERENCES mod_lock_comment ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE modlog_combined
    DROP CONSTRAINT modlog_combined_check,
    ADD CONSTRAINT modlog_combined_check CHECK (num_nonnulls (admin_allow_instance_id, admin_block_instance_id, admin_purge_comment_id, admin_purge_community_id, admin_purge_person_id, admin_purge_post_id, admin_add_id, mod_add_to_community_id, admin_ban_id, mod_ban_from_community_id, mod_feature_post_id, mod_change_community_visibility_id, mod_lock_post_id, mod_remove_comment_id, admin_remove_community_id, mod_remove_post_id, mod_transfer_community_id, mod_lock_comment_id) = 1),
    ALTER CONSTRAINT modlog_combined_mod_lock_comment_id_fkey NOT DEFERRABLE;

