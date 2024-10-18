-- replace value of inbox_url with shared_inbox_url and the drop shared inbox
UPDATE
    person
SET
    inbox_url = subquery.inbox_url
FROM (
    SELECT
        id,
        coalesce(shared_inbox_url, inbox_url) AS inbox_url
    FROM
        person) AS subquery
WHERE
    person.id = subquery.id;

ALTER TABLE person
    DROP COLUMN shared_inbox_url;

UPDATE
    community
SET
    inbox_url = subquery.inbox_url
FROM (
    SELECT
        id,
        coalesce(shared_inbox_url, inbox_url) AS inbox_url
    FROM
        community) AS subquery
WHERE
    community.id = subquery.id;

ALTER TABLE community
    DROP COLUMN shared_inbox_url;

