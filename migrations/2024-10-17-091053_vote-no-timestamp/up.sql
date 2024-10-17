-- set all column values to null to reclaim disk space
-- https://dba.stackexchange.com/a/117513
ALTER TABLE post_like
    ALTER COLUMN published DROP NOT NULL;

UPDATE
    post_like
SET
    published = NULL;

ALTER TABLE comment_like
    ALTER COLUMN published DROP NOT NULL;

UPDATE
    comment_like
SET
    published = NULL;

ALTER TABLE comment_like
    ALTER COLUMN post_id DROP NOT NULL;

UPDATE
    post_like
SET
    post_id = NULL;

-- drop the columns
ALTER TABLE post_like
    DROP published;

ALTER TABLE comment_like
    DROP published;

ALTER TABLE comment_like
    DROP post_id;

