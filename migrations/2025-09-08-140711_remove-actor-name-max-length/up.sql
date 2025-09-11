-- get rid of max name length setting
ALTER TABLE local_site
    DROP COLUMN actor_name_max_length;

-- truncate existing strings
UPDATE
    person
SET
    display_name = substring(display_name FROM 1 FOR 50)
WHERE
    length(display_name) > 50;

UPDATE
    community
SET
    title = substring(title FROM 1 FOR 50)
WHERE
    length(title) > 50;

-- reduce max length of db columns
ALTER TABLE person
    ALTER COLUMN display_name TYPE varchar(50);

ALTER TABLE community
    ALTER COLUMN title TYPE varchar(50);

