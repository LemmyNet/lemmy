ALTER TABLE post
    ADD COLUMN instance_id int NOT NULL DEFAULT 0 REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE;

-- Update the historical rows
UPDATE
    post AS p
SET
    instance_id = c.instance_id
FROM
    community AS c
WHERE
    p.community_id = c.id;

