-- Add ban_expires to person, community_person_ban
ALTER TABLE person
    ADD COLUMN ban_expires timestamp;

ALTER TABLE community_person_ban
    ADD COLUMN expires timestamp;

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

