ALTER TABLE community RENAME description TO summary;

ALTER TABLE community RENAME COLUMN sidebar TO description;

alter table community_report rename original_community_description to original_community_summary;

alter table community_report rename original_community_sidebar to original_community_description;