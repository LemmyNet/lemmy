-- lemmy requires (username + instance_id) to be unique
-- delete any existing duplicates
DELETE FROM person p1 USING (
    SELECT
        min(id) AS id,
        name,
        instance_id
    FROM
        person
    GROUP BY
        name,
        instance_id
    HAVING
        count(*) > 1) p2
WHERE
    p1.name = p2.name
    AND p1.instance_id = p2.instance_id
    AND p1.id <> p2.id;

ALTER TABLE person
    ADD CONSTRAINT person_name_instance_unique UNIQUE (name, instance_id);

