-- replace value of inbox_url with shared_inbox_url and the drop shared inbox
UPDATE
    person
SET
    shared_inbox_url = inbox_url
WHERE
    shared_inbox_url IS NULL;

ALTER TABLE person
    DROP COLUMN inbox_url,
    ALTER COLUMN shared_inbox_url SET NOT NULL,
    ALTER COLUMN shared_inbox_url SET DEFAULT generate_unique_changeme ();

ALTER TABLE person RENAME COLUMN shared_inbox_url TO inbox_url;

UPDATE
    community
SET
    shared_inbox_url = inbox_url
WHERE
    shared_inbox_url IS NULL;

ALTER TABLE community
    DROP COLUMN inbox_url,
    ALTER COLUMN shared_inbox_url SET NOT NULL,
    ALTER COLUMN shared_inbox_url SET DEFAULT generate_unique_changeme ();

ALTER TABLE community RENAME COLUMN shared_inbox_url TO inbox_url;

