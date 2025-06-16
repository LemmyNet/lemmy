-- Remove the comment.read column, and create a new comment_reply table,
-- similar to the person_mention table.
--
-- This is necessary because self-joins using ltrees would be too tough with SQL views
--
-- Every comment should have a row here, because all comments have a recipient,
-- either the post creator, or the parent commenter.
CREATE TABLE comment_reply (
    id serial PRIMARY KEY,
    recipient_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    read boolean DEFAULT FALSE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (recipient_id, comment_id)
);

-- Ones where parent_id is null, use the post creator recipient
INSERT INTO comment_reply (recipient_id, comment_id, read)
SELECT
    p.creator_id,
    c.id,
    c.read
FROM
    comment c
    INNER JOIN post p ON c.post_id = p.id
WHERE
    c.parent_id IS NULL;

--  Ones where there is a parent_id, self join to comment to get the parent comment creator
INSERT INTO comment_reply (recipient_id, comment_id, read)
SELECT
    c2.creator_id,
    c.id,
    c.read
FROM
    comment c
    INNER JOIN comment c2 ON c.parent_id = c2.id;

-- Drop comment_alias view
DROP VIEW comment_alias_1;

ALTER TABLE comment
    DROP COLUMN read;

CREATE EXTENSION IF NOT EXISTS ltree;

ALTER TABLE comment
    ADD COLUMN path ltree NOT NULL DEFAULT '0';

ALTER TABLE comment_aggregates
    ADD COLUMN child_count integer NOT NULL DEFAULT 0;

-- The ltree path column should be the comment_id parent paths, separated by dots.
-- Stackoverflow: building an ltree from a parent_id hierarchical tree:
-- https://stackoverflow.com/a/1144848/1655478
CREATE TEMPORARY TABLE comment_temp AS
WITH RECURSIVE q AS (
    SELECT
        h,
        1 AS level,
        ARRAY[id] AS breadcrumb
    FROM
        comment h
    WHERE
        parent_id IS NULL
    UNION ALL
    SELECT
        hi,
        q.level + 1 AS level,
        breadcrumb || id
    FROM
        q
        JOIN comment hi ON hi.parent_id = (q.h).id
)
SELECT
    (q.h).id,
    (q.h).parent_id,
    level,
    breadcrumb::varchar AS path,
    text2ltree ('0.' || array_to_string(breadcrumb, '.')) AS ltree_path
FROM
    q
ORDER BY
    breadcrumb;

-- Remove indexes and foreign key constraints, and disable triggers for faster updates
ALTER TABLE comment DISABLE TRIGGER USER;

ALTER TABLE comment
    DROP CONSTRAINT IF EXISTS comment_creator_id_fkey;

ALTER TABLE comment
    DROP CONSTRAINT IF EXISTS comment_parent_id_fkey;

ALTER TABLE comment
    DROP CONSTRAINT IF EXISTS comment_post_id_fkey;

ALTER TABLE comment
    DROP CONSTRAINT IF EXISTS idx_comment_ap_id;

DROP INDEX IF EXISTS idx_comment_creator;

DROP INDEX IF EXISTS idx_comment_parent;

DROP INDEX IF EXISTS idx_comment_post;

DROP INDEX IF EXISTS idx_comment_published;

-- Add the ltree column
UPDATE
    comment c
SET
    path = ct.ltree_path
FROM
    comment_temp ct
WHERE
    c.id = ct.id;

-- Without this, `DROP EXTENSION` in down.sql throws an object dependency error if up.sql and down.sql
-- are run in the same database connection
DROP TABLE comment_temp;

-- Update the child counts
UPDATE
    comment_aggregates ca
SET
    child_count = c2.child_count
FROM (
    SELECT
        c.id,
        c.path,
        count(c2.id) AS child_count
    FROM
        comment c
    LEFT JOIN comment c2 ON c2.path <@ c.path
        AND c2.path != c.path
GROUP BY
    c.id) AS c2
WHERE
    ca.comment_id = c2.id;

-- Delete comments at a depth of > 150, otherwise the index creation below will fail
DELETE FROM comment
WHERE nlevel (path) > 150;

-- Delete from comment where there is a missing post
DELETE FROM comment c
WHERE NOT EXISTS (
        SELECT
        FROM
            post p
        WHERE
            p.id = c.post_id);

-- Delete from comment where there is a missing creator_id
DELETE FROM comment c
WHERE NOT EXISTS (
        SELECT
        FROM
            person p
        WHERE
            p.id = c.creator_id);

-- Re-enable old constraints and indexes
ALTER TABLE comment
    ADD CONSTRAINT "comment_creator_id_fkey" FOREIGN KEY (creator_id) REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE comment
    ADD CONSTRAINT "comment_post_id_fkey" FOREIGN KEY (post_id) REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE comment
    ADD CONSTRAINT "idx_comment_ap_id" UNIQUE (ap_id);

CREATE INDEX idx_comment_creator ON comment (creator_id);

CREATE INDEX idx_comment_post ON comment (post_id);

CREATE INDEX idx_comment_published ON comment (published DESC);

-- Create the index
CREATE INDEX idx_path_gist ON comment USING gist (path);

-- Drop the parent_id column
ALTER TABLE comment
    DROP COLUMN parent_id CASCADE;

ALTER TABLE comment ENABLE TRIGGER USER;

