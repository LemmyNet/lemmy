ALTER TABLE local_user
    ADD COLUMN matrix_user_id text;

ALTER TABLE local_user
    ADD COLUMN admin boolean DEFAULT FALSE NOT NULL;

UPDATE
    local_user lu
SET
    matrix_user_id = p.matrix_user_id,
    admin = p.admin
FROM
    person p
WHERE
    p.id = lu.person_id;

DROP VIEW person_alias_1, person_alias_2;

ALTER TABLE person
    DROP COLUMN matrix_user_id;

ALTER TABLE person
    DROP COLUMN admin;

-- Regenerate the person_alias views
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

