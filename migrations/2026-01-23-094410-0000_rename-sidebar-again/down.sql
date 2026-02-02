ALTER TABLE community RENAME sidebar TO description;

ALTER TABLE community_report RENAME original_community_sidebar TO original_community_description;

ALTER TABLE site RENAME sidebar TO description;

ALTER TABLE multi_community RENAME summary TO description;

ALTER TABLE tag RENAME summary TO description;

ALTER TABLE tag
    ALTER description TYPE text;

