-- Creates combined tables for
-- Search: (post, comment, community, person)
-- Add published to person_aggregates (it was missing for some reason)
ALTER TABLE person_aggregates
    ADD COLUMN published timestamptz NOT NULL DEFAULT now();

UPDATE
    person_aggregates pa
SET
    published = p.published
FROM
    person p
WHERE
    pa.person_id = p.id;

