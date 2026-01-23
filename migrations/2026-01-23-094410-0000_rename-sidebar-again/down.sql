ALTER TABLE community RENAME sidebar TO description;

ALTER TABLE multi_community RENAME sidebar TO description;

ALTER TABLE site RENAME sidebar TO description;

ALTER TABLE tag RENAME summary TO description;

ALTER TABLE tag
    ALTER description TYPE text;

