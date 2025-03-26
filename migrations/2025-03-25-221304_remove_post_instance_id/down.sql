ALTER TABLE post
    ADD COLUMN instance_id int DEFAULT 0 NOT NULL REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED;

-- Update the historical rows
UPDATE
    post AS p
SET
    instance_id = c.instance_id
FROM
    community AS c
WHERE
    p.community_id = c.id;

