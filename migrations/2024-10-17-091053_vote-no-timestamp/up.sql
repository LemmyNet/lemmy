-- make published columns nullable and remove default value
ALTER TABLE post_like
    ALTER COLUMN published DROP NOT NULL;

ALTER TABLE post_like
    ALTER COLUMN published DROP DEFAULT;

ALTER TABLE comment_like
    ALTER COLUMN published DROP NOT NULL;

ALTER TABLE comment_like
    ALTER COLUMN published DROP DEFAULT;

-- get rid of comment_like.post_id, setting null first to reclaim space
-- https://dba.stackexchange.com/a/117513
ALTER TABLE comment_like
    ALTER COLUMN post_id DROP NOT NULL;

UPDATE
    post_like
SET
    post_id = NULL;

-- drop the columns
ALTER TABLE comment_like
    DROP post_id;

