-- Remove text fields inbox_url, shared_inbox_url from person, community and site.
-- Instead use a foreign key for these urls so they can be deduplicated.
-- New inbox table
-- TODO: add trigger which removes unused inbox items
CREATE TABLE inbox (
    id serial PRIMARY KEY,
    url varchar(255) NOT NULL
);

-- Move existing inbox values to inbox table, and replace with foreign key
ALTER TABLE person
    ADD COLUMN inbox_id int;

WITH inboxes AS (
    SELECT
        id AS person_id,
        coalesce(shared_inbox_url, inbox_url) AS url
    FROM
        person
),
inserted AS (
INSERT INTO inbox (url)
    SELECT
        url
    FROM
        inboxes
    ON CONFLICT
        DO NOTHING
    RETURNING
        id,
        url)
UPDATE
    person
SET
    inbox_id = inserted.id
FROM
    inboxes,
    inserted
WHERE
    person.id = inboxes.person_id
    AND inserted.url = inboxes.url;

ALTER TABLE person
    ADD CONSTRAINT person_inbox_id_fkey FOREIGN KEY (inbox_id) REFERENCES inbox (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE person
    ALTER COLUMN inbox_id SET NOT NULL;

-- Drop old columns and rename new one
ALTER TABLE person
    DROP COLUMN inbox_url;

ALTER TABLE person
    DROP COLUMN shared_inbox_url;

-- TODO: same thing for community and site
