DROP VIEW person_alias_1, person_alias_2;

ALTER TABLE person
    DROP COLUMN bot_account;

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

ALTER TABLE local_user
    DROP COLUMN show_bot_accounts;

