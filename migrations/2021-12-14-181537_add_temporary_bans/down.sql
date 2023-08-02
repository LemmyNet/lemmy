DROP VIEW person_alias_1, person_alias_2;

ALTER TABLE person
    DROP COLUMN ban_expires;

ALTER TABLE community_person_ban
    DROP COLUMN expires;

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

