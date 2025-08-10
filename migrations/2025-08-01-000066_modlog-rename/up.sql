ALTER TABLE mod_ban RENAME TO admin_ban;

ALTER TABLE mod_add RENAME TO admin_add;

ALTER TABLE mod_remove_community RENAME TO admin_remove_community;

ALTER TABLE mod_add_community RENAME TO mod_add_to_community;

ALTER TABLE modlog_combined RENAME COLUMN mod_ban_id TO admin_ban_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_add_id TO admin_add_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_remove_community_id TO admin_remove_community_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_add_community_id TO mod_add_to_community_id;

