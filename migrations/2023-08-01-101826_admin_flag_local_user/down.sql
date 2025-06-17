ALTER TABLE person
    ADD COLUMN admin boolean DEFAULT FALSE NOT NULL;

UPDATE
    person
SET
    admin = TRUE
FROM
    local_user
WHERE
    local_user.person_id = person.id
    AND local_user.admin;

ALTER TABLE local_user
    DROP COLUMN admin;

CREATE INDEX idx_person_admin ON person (admin)
WHERE
    admin;

