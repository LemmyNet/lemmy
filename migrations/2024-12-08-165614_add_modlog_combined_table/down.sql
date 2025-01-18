DROP TABLE modlog_combined;

-- Rename the columns back to when_
ALTER TABLE admin_allow_instance RENAME COLUMN published TO when_;

ALTER TABLE admin_block_instance RENAME COLUMN published TO when_;

ALTER TABLE admin_purge_comment RENAME COLUMN published TO when_;

ALTER TABLE admin_purge_community RENAME COLUMN published TO when_;

ALTER TABLE admin_purge_person RENAME COLUMN published TO when_;

ALTER TABLE admin_purge_post RENAME COLUMN published TO when_;

ALTER TABLE mod_add RENAME COLUMN published TO when_;

ALTER TABLE mod_add_community RENAME COLUMN published TO when_;

ALTER TABLE mod_ban RENAME COLUMN published TO when_;

ALTER TABLE mod_ban_from_community RENAME COLUMN published TO when_;

ALTER TABLE mod_feature_post RENAME COLUMN published TO when_;

ALTER TABLE mod_hide_community RENAME COLUMN published TO when_;

ALTER TABLE mod_lock_post RENAME COLUMN published TO when_;

ALTER TABLE mod_remove_comment RENAME COLUMN published TO when_;

ALTER TABLE mod_remove_community RENAME COLUMN published TO when_;

ALTER TABLE mod_remove_post RENAME COLUMN published TO when_;

ALTER TABLE mod_transfer_community RENAME COLUMN published TO when_;

