ALTER TABLE admin_ban RENAME TO mod_ban;

ALTER TABLE admin_add RENAME TO mod_add;

ALTER TABLE admin_remove_community RENAME TO mod_remove_community;

ALTER TABLE mod_add_to_community RENAME TO mod_add_community;

ALTER TABLE modlog_combined RENAME COLUMN admin_ban_id TO mod_ban_id;

ALTER TABLE modlog_combined RENAME COLUMN admin_add_id TO mod_add_id;

ALTER TABLE modlog_combined RENAME COLUMN admin_remove_community_id TO mod_remove_community_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_add_to_community_id TO mod_add_community_id;

