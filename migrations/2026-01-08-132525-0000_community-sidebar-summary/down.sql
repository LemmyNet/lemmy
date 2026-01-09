ALTER TABLE community RENAME COLUMN description TO sidebar;

ALTER TABLE community RENAME summary TO description;

alter table community_report rename original_community_description  to original_community_sidebar;

alter table community_report rename original_community_summary to  original_community_description;
