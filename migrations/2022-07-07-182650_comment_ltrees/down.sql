ALTER TABLE comment
    ADD COLUMN parent_id integer;

-- Constraints and index
ALTER TABLE comment
    ADD CONSTRAINT comment_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES comment (id) ON UPDATE CASCADE ON DELETE CASCADE;

CREATE INDEX idx_comment_parent ON comment (parent_id);

-- Update the parent_id column
-- subpath(subpath(0, -1), -1) gets the immediate parent but it fails null checks
UPDATE
    comment
SET
    parent_id = cast(ltree2text (nullif (subpath (nullif (subpath (path, 0, -1), '0'), -1), '0')) AS INTEGER);

ALTER TABLE comment
    DROP COLUMN path;

ALTER TABLE comment_aggregates
    DROP COLUMN child_count;

DROP EXTENSION ltree;

-- Add back in the read column
ALTER TABLE comment
    ADD COLUMN read boolean DEFAULT FALSE NOT NULL;

UPDATE
    comment c
SET
    read = cr.read
FROM
    comment_reply cr
WHERE
    cr.comment_id = c.id;

CREATE VIEW comment_alias_1 AS
SELECT
    *
FROM
    comment;

DROP TABLE comment_reply;

