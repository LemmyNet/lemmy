ALTER TABLE community RENAME COLUMN description TO sidebar;

ALTER TABLE community RENAME summary TO description;

ALTER TABLE community_report RENAME original_community_description TO original_community_sidebar;

ALTER TABLE community_report RENAME original_community_summary TO original_community_description;

ALTER TABLE site RENAME COLUMN description TO sidebar;

ALTER TABLE site RENAME summary TO description;

