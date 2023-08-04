ALTER TABLE person
    ADD COLUMN matrix_user_id text;

ALTER TABLE person
    ADD COLUMN admin boolean DEFAULT FALSE NOT NULL;

UPDATE
    person p
SET
    matrix_user_id = lu.matrix_user_id,
    admin = lu.admin
FROM
    local_user lu
WHERE
    p.id = lu.person_id;

ALTER TABLE local_user
    DROP COLUMN matrix_user_id;

ALTER TABLE local_user
    DROP COLUMN admin;

-- Regenerate the person_alias views
DROP VIEW person_alias_1, person_alias_2;

CREATE VIEW person_alias_1 AS
SELECT
    *
FROM
    person;

CREATE VIEW person_alias_2 AS
SELECT
    *
FROM
    person;

