-- Add the bot_account column to the person table
DROP VIEW person_alias_1, person_alias_2;

ALTER TABLE person
    ADD COLUMN bot_account boolean NOT NULL DEFAULT FALSE;

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

-- Add the show_bot_accounts to the local user table as a setting
ALTER TABLE local_user
    ADD COLUMN show_bot_accounts boolean NOT NULL DEFAULT TRUE;

