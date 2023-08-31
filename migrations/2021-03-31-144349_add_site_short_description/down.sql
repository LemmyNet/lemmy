ALTER TABLE site
    DROP COLUMN description;

ALTER TABLE site RENAME COLUMN sidebar TO description;

