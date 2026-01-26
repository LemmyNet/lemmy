ALTER TABLE community RENAME description TO sidebar;

ALTER TABLE community_report RENAME original_community_description TO original_community_sidebar;

ALTER TABLE site RENAME description TO sidebar;

-- using summary for this because it has 150 char limit
ALTER TABLE multi_community RENAME description TO summary;

ALTER TABLE tag RENAME description TO summary;

ALTER TABLE tag
    ALTER summary TYPE varchar(150);

