ALTER TABLE community RENAME description TO summary;

ALTER TABLE community RENAME COLUMN sidebar TO description;

ALTER TABLE community_report RENAME original_community_description TO original_community_summary;

ALTER TABLE community_report RENAME original_community_sidebar TO original_community_description;

ALTER TABLE site RENAME description TO summary;

ALTER TABLE site RENAME COLUMN sidebar TO description;

