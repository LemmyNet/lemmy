DROP VIEW person_alias_1;

DROP VIEW person_alias_2;

ALTER TABLE community
    ALTER COLUMN name TYPE varchar(20);

ALTER TABLE community
    ALTER COLUMN title TYPE varchar(100);

ALTER TABLE person
    ALTER COLUMN name TYPE varchar(20);

ALTER TABLE person
    ALTER COLUMN display_name TYPE varchar(20);

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

