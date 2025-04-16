UPDATE
    mod_ban
SET
    instance_id = person.instance_id
FROM
    person
WHERE
    mod_ban.instance_id IS NULL
    AND mod_ban.mod_person_id = person.id;

ALTER TABLE mod_ban
    ALTER COLUMN instance_id SET NOT NULL;

