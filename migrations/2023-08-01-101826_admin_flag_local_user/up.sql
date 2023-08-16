ALTER TABLE local_user
    ADD COLUMN admin boolean DEFAULT FALSE NOT NULL;

UPDATE
    local_user
SET
    admin = TRUE
FROM
    person
WHERE
    local_user.person_id = person.id
    AND person.admin;

ALTER TABLE person
    DROP COLUMN admin;

