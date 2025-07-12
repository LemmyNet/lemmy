-- lemmy requires (username + instance_id) to be unique
-- delete any existing duplicates
DELETE FROM person p1 USING person p2
WHERE p1.id > p2.id
    AND p1.name = p2.name
    AND p1.instance_id = p2.instance_id;

ALTER TABLE person
    ADD CONSTRAINT person_name_instance_unique UNIQUE (name, instance_id);

