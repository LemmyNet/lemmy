ALTER TABLE person RENAME display_name TO preferred_username;

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

