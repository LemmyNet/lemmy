ALTER TABLE modlog_combined
    DROP COLUMN mod_lock_comment_id,
    ADD CONSTRAINT modlog_combined_check CHECK (num_nonnulls (admin_allow_instance_id, admin_block_instance_id, admin_purge_comment_id, admin_purge_community_id, admin_purge_person_id, admin_purge_post_id, admin_add_id, mod_add_to_community_id, admin_ban_id, mod_ban_from_community_id, mod_feature_post_id, mod_change_community_visibility_id, mod_lock_post_id, mod_remove_comment_id, admin_remove_community_id, mod_remove_post_id, mod_transfer_community_id) = 1);

DROP TABLE mod_lock_comment;

ALTER TABLE comment
    DROP COLUMN LOCKED;

