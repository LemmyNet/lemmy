ALTER TABLE notification
    ADD COLUMN admin_add_id int REFERENCES admin_add ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_add_to_community_id int REFERENCES mod_add_to_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN admin_ban_id int REFERENCES admin_ban ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_ban_from_community_id int REFERENCES mod_ban_from_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_feature_post_id int REFERENCES mod_feature_post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_lock_post_id int REFERENCES mod_lock_post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_remove_comment_id int REFERENCES mod_remove_comment ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN admin_remove_community_id int REFERENCES admin_remove_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_remove_post_id int REFERENCES mod_remove_post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_transfer_community_id int REFERENCES mod_transfer_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_lock_comment_id int REFERENCES mod_lock_comment ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TYPE notification_type_enum
    ADD value 'ModAction';

ALTER TYPE notification_type_enum
    ADD value 'RevertModAction';

