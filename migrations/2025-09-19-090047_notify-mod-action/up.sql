-- new foreign keys
ALTER TABLE notification
    ADD COLUMN admin_add_id int REFERENCES admin_add ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_add_to_community_id int REFERENCES mod_add_to_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN admin_ban_id int REFERENCES admin_ban ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_ban_from_community_id int REFERENCES mod_ban_from_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_lock_post_id int REFERENCES mod_lock_post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_remove_comment_id int REFERENCES mod_remove_comment ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN admin_remove_community_id int REFERENCES admin_remove_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_remove_post_id int REFERENCES mod_remove_post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_lock_comment_id int REFERENCES mod_lock_comment ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_transfer_community_id int REFERENCES mod_transfer_community ON UPDATE CASCADE ON DELETE CASCADE;

-- new types for mod actions
ALTER TYPE notification_type_enum
    ADD value 'ModAction';

-- update constraint with new columns
ALTER TABLE notification
    DROP CONSTRAINT IF EXISTS notification_check;

ALTER TABLE notification
    ADD CONSTRAINT notification_check CHECK (num_nonnulls (post_id, comment_id, private_message_id, admin_add_id, mod_add_to_community_id, admin_ban_id, mod_ban_from_community_id, mod_lock_post_id, mod_remove_post_id, mod_lock_comment_id, mod_remove_comment_id, admin_remove_community_id, mod_transfer_community_id) = 1);

-- add indexes
CREATE INDEX idx_notification_unread ON notification (read);

CREATE INDEX idx_notification_admin_add_id ON notification (admin_add_id)
WHERE
    admin_add_id IS NOT NULL;

CREATE INDEX idx_notification_mod_add_to_community_id ON notification (mod_add_to_community_id)
WHERE
    mod_add_to_community_id IS NOT NULL;

CREATE INDEX idx_notification_admin_ban_id ON notification (admin_ban_id)
WHERE
    admin_ban_id IS NOT NULL;

CREATE INDEX idx_notification_mod_ban_from_community_id ON notification (mod_ban_from_community_id)
WHERE
    mod_ban_from_community_id IS NOT NULL;

CREATE INDEX idx_notification_mod_lock_post_id ON notification (mod_lock_post_id)
WHERE
    mod_lock_post_id IS NOT NULL;

CREATE INDEX idx_notification_mod_remove_comment_id ON notification (mod_remove_comment_id)
WHERE
    mod_remove_comment_id IS NOT NULL;

CREATE INDEX idx_notification_admin_remove_community_id ON notification (admin_remove_community_id)
WHERE
    admin_remove_community_id IS NOT NULL;

CREATE INDEX idx_notification_mod_remove_post_id ON notification (mod_remove_post_id)
WHERE
    mod_remove_post_id IS NOT NULL;

CREATE INDEX idx_notification_mod_lock_comment_id ON notification (mod_lock_comment_id)
WHERE
    mod_lock_comment_id IS NOT NULL;

CREATE INDEX idx_notification_mod_transfer_community_id ON notification (mod_transfer_community_id)
WHERE
    mod_transfer_community_id IS NOT NULL;

