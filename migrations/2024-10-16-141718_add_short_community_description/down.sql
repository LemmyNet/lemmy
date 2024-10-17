ALTER TABLE community
    DROP COLUMN description;

ALTER TABLE community RENAME COLUMN sidebar TO description;

