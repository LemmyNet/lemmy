--  Add the column back
ALTER TABLE site
    ADD COLUMN creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE;

-- Add the data, selecting the highest admin
UPDATE
    site
SET
    creator_id = sub.id
FROM (
    SELECT
        id
    FROM
        person
    WHERE
        admin = TRUE
    LIMIT 1) AS sub;

-- Set to not null
ALTER TABLE site
    ALTER COLUMN creator_id SET NOT NULL;

